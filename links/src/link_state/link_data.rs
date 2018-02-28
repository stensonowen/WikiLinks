
use {slog, fst, serde_json};
use fnv::FnvHashMap;

use std::io::{self, Read, Write, BufRead, BufReader};
use std::path::PathBuf;
use std::fs::File;
use std::ffi::OsString;
use std::thread;

use super::{LinkState, LinkDb, LinkData};


/// A wiki entry, including its title, parents, and children
/// The parent and child sets have a fair amount of overlap, so instead of storing both they're
/// both put in the `neighbors` field (parents first, then children), of which both parents and
/// children are subsets
/// Note that the indices are stored as `u16`s meaning if an entry has more than 65k parents 
/// or children then it will cause problems.
#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub title: String,
    pub id: u32,
    neighbors: Vec<u32>,
    last_parent: u32,
    first_child: u32,
}

impl Entry {
    #[inline]
    pub fn get_children(&self) -> &[u32] {
        let i = self.first_child as usize;
        &self.neighbors[i..]
    }
    #[inline]
    pub fn get_parents(&self) -> &[u32] {
        let i = self.last_parent as usize;
        &self.neighbors[..i]
    }
}



impl Entry {
    pub fn from(i: u32, t: String, parents: Vec<u32>, children: Vec<u32>) -> Self {
        use std::collections::HashSet;
        let parent_set: HashSet<u32> = parents.iter().cloned().collect();
        assert_eq!(parent_set.len(), parents.len(), "Entry `{}`", t);
        assert!(parents.len() < u32::max_value() as usize, 
                "Entry `{}` has {} parents", t, parents.len());
        let child_set: HashSet<u32> = children.iter().cloned().collect();
        assert_eq!(child_set.len(), children.len(), "Entry `{}`", t);
        assert!(children.len() < u32::max_value() as usize, 
                "Entry `{}` has {} children", t, children.len());
        let last_parent = parents.len() as u32;
        let num_children = children.len();
        let parents_hm: HashSet<u32> = parents.iter().cloned().collect();
        let common: HashSet<u32> = children.iter().cloned().filter(|i| {
            parents_hm.contains(i)
        }).collect();
        let unique_pars =  parents.into_iter().filter(|i| !common.contains(i));
        let unique_kids = children.into_iter().filter(|i| !common.contains(i));
        let neighbors: Vec<u32> = unique_pars
            .chain(common.iter().cloned())
            .chain(unique_kids).collect();
        let first_child = (neighbors.len() - num_children) as u32;

        Entry {
            id: i,
            title: t,
            neighbors, last_parent, first_child
        }
    }
}

impl From<LinkState<LinkDb>> for LinkState<LinkData> {
    fn from(old: LinkState<LinkDb>) -> LinkState<LinkData> {
        // entries will become into lookup table
        // addresses and ranks feed into PostgreSQL
        
        let (entries_i, titles) = old.state.parts();
        let mut entries: Vec<Vec<Entry>> = Vec::with_capacity(old.threads);

        // convert `titles` to a fst Map
        // for now, do this in memory (can slightly better optimize or something)
        let mut titles_sorted = titles.clone().into_iter().map(|(title,id)| {
            (title, u64::from(id))
        }).collect::<Vec<(String,u64)>>();
        titles_sorted.sort_by(|a,b| a.0.cmp(&b.0));
        let mut mb = fst::MapBuilder::memory();
        mb.extend_iter(titles_sorted.into_iter()).expect("fst population");
        let fst_bytes = mb.into_inner().expect("fst finilize");

        //seems like there should be a more functional way to do this
        //  if .take() didn't consume?  doesn't shallow copy??
        // could stand to be refactored
        let size = old.size / old.threads + 1;
        for _ in 0 .. old.threads+1 {
            entries.push(Vec::with_capacity(size));
        }
        let mut count = 0usize;
        for entry in entries_i {
            entries[count/size].push(entry);
            count += 1;
        }
        assert_eq!(count, old.size, "Lost elements populating LinkDb");

        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      LinkData {
                dumps:  entries,
                titles: fst_bytes,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkManifest {
    threads: usize,
    size:    usize,
    entries: Vec<PathBuf>,
    titles:  PathBuf,
}

pub fn append_to_pathbuf(p: &PathBuf, addition: &str, extension: &str) -> PathBuf {
    let mut name = OsString::from(p.file_stem().unwrap());
    name.push(OsString::from(addition));
    PathBuf::from(p).with_file_name(name).with_extension(extension)
}


impl LinkState<LinkData> {
    // need to read from or write to files to restore from/to this state
    fn manifest(&self, mn: &PathBuf) -> LinkManifest {
        LinkManifest {
            threads:    self.threads,
            size:       self.size,
            titles:      append_to_pathbuf(mn, "_titles", "fst"),
            entries:    (0..self.threads).map(|i| {
                let mut name = String::from("_entry");
                name.push_str(&i.to_string());
                append_to_pathbuf(mn, &name, "json")
            }).collect(),
        }
    }
    pub fn break_down(self) -> (FnvHashMap<u32,Entry>, slog::Logger, Vec<u8>) {
        let mut hm: FnvHashMap<u32,Entry> = 
            FnvHashMap::with_capacity_and_hasher(self.size, Default::default());
        for bucket in self.state.dumps {
            for ie in bucket {
                let id = ie.id;
                let entry: Entry = ie.into();
                hm.insert(id, entry);
            }
        }
        (hm, self.log, self.state.titles)
    }
    pub fn export(&self, dst: PathBuf) -> io::Result<()> {
        // write output to line-delimited JSON and CSV types
        let manifest = self.manifest(&dst);
        //write manifest
        let mut mn_f = File::create(dst)?;
        let mn_s = serde_json::to_string(&manifest).expect("serialize manifest");
        mn_f.write_all(&mn_s.into_bytes())?;
        
        println!("Manifest: `{:?}`", manifest);

        // write title bytes (to be mmapped/opened later)
        let title_f = File::create(manifest.titles)?;
        let mut title_w = io::BufWriter::new(title_f);
        title_w.write_all(&self.state.titles)?;

        //write entries to `self.threads` different files
        for (i,p) in manifest.entries.iter().enumerate() {
            println!("Writing to `{:?}`", p);
            let mut f = File::create(p)?;
            let dump = &self.state.dumps[i];
            for i in dump {
                let mut serial = serde_json::to_string(i).expect("serialize entry");
                serial.push('\n');
                f.write_all(&serial.into_bytes())?;
            }
        }
        Ok(()) 
    }


    pub fn import(src: PathBuf, log: slog::Logger) -> Result<Self,io::Error> { 
        assert!(src.is_file());
        let mut s = String::new();
        File::open(src).and_then(|mut f: File| f.read_to_string(&mut s))?;
        let manifest: LinkManifest = serde_json::from_str(&s).unwrap();

        // populate titles
        // for now just copy into memory and convert later
        // in the future mmapping might be cool, but I don't think it's super important
        // for now I'd prefer to just avoid unsafe :), even if it could maybe
        //  save ~100Mb of RAM (and I want consistently good performance)
        let titles_f = File::open(&manifest.titles)?;
        let mut titles_br = io::BufReader::new(titles_f);
        let mut titles_b = vec![];
        titles_br.read_to_end(&mut titles_b)?;

        let threads = manifest.entries.into_iter().map(|p| {
            thread::spawn(move || {
                let f = File::open(p)?;
                let r = BufReader::new(f);
                let err = "Deserializing data";
                r.lines().map(|l| l.map(|s| serde_json::from_str(&s).expect(err)))
                    .collect::<io::Result<Vec<Entry>>>()
            })
        }).collect::<Vec<thread::JoinHandle<io::Result<Vec<Entry>>>>>();
        // handle these unwraps better? look into `failure`?
        let data: Vec<Vec<Entry>> = threads.into_iter()
            .map(|t| t.join().unwrap().unwrap())
            .collect();

        Ok(LinkState {
            log,
            threads: manifest.threads,
            size:    manifest.size,
            state:   LinkData {
                dumps: data,
                titles: titles_b,
            }
        })
    }
}

impl LinkData {
    pub fn consolidate_links(links: Vec<Vec<Entry>>, size: usize) 
        -> FnvHashMap<u32,Entry> 
    {
        let mut hm: FnvHashMap<u32,Entry> = 
            FnvHashMap::with_capacity_and_hasher(size, Default::default());
        for bucket in links {
            for ie in bucket {
                let id = ie.id;
                let entry: Entry = ie.into();
                hm.insert(id, entry);
            }
        }
        hm
    }
}
