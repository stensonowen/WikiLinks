
use {slog, csv, serde_json};
use fnv::FnvHashMap;
//use std::sync::Mutex;
use std::io::{self, Read, Write, BufRead, BufReader};
use std::path::PathBuf;
use std::fs::File;
use std::ffi::OsString;
use std::collections::HashMap;

use super::{LinkState, LinkDb, LinkData};
use super::Entry;

use std::thread;

// TODO replace IndexedEntry with (u32, Entry) ?
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedEntry {
    pub id: u32,
    pub title: String,
    // See representation of Entry
    pub neighbors: Vec<u32>,
    pub last_parent: u32,
    pub first_child: u32,
}

impl IndexedEntry {
    pub fn from(i: u32, t: String, parents: Vec<u32>, children: Vec<u32>) -> Self {
        use std::collections::HashSet;
        let parent_set: HashSet<u32> = parents.iter().map(|&i| i).collect();
        assert_eq!(parent_set.len(), parents.len(), "Entry `{}`", t);
        assert!(parents.len() < u32::max_value() as usize, 
                "Entry `{}` has {} parents", t, parents.len());
        let child_set: HashSet<u32> = children.iter().map(|&i| i).collect();
        assert_eq!(child_set.len(), children.len(), "Entry `{}`", t);
        assert!(children.len() < u32::max_value() as usize, 
                "Entry `{}` has {} children", t, children.len());
        let last_parent = parents.len() as u32;
        let num_children = children.len();
        let parents_hm: HashSet<u32> = parents.iter().map(|&i| i).collect();
        let common: HashSet<u32> = children.iter().map(|&i| i).filter(|i| {
            parents_hm.contains(&i)
        }).collect();
        let unique_pars =  parents.into_iter().filter(|i| !common.contains(&i));
        let unique_kids = children.into_iter().filter(|i| !common.contains(&i));
        let neighbors: Vec<u32> = unique_pars
            .chain(common.iter().map(|&i| i))
            .chain(unique_kids).collect();
        let first_child = (neighbors.len() - num_children) as u32;

        IndexedEntry {
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
        //let mut entries: Vec<Mutex<Vec<IndexedEntry>>> = Vec::with_capacity(old.threads);
        let mut entries: Vec<Vec<IndexedEntry>> = Vec::with_capacity(old.threads);

        //seems like there should be a more functional way to do this
        //  if .take() didn't consume?  doesn't shallow copy??
        // could stand to be refactored
        let size = old.size / old.threads + 1;
        for _ in 0..old.threads+1 {
            //entries.push(Mutex::new(Vec::with_capacity(size)));
            entries.push(Vec::with_capacity(size));
        }
        let mut count = 0usize;
        for entry in entries_i {
            //entries[count/size].get_mut().unwrap().push(entry);
            entries[count/size].push(entry);
            count += 1;
        }
        assert_eq!(count, old.size, "Lost elements populating LinkDb");

        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      LinkData {
                dumps: entries,
                titles: titles,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkManifest {
    threads: usize,
    size:    usize,
    entries: Vec<PathBuf>,
    titles:   PathBuf,
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
            titles:      append_to_pathbuf(mn, "_titles", "csv"),
            entries:    (0..self.threads).map(|i| {
                let mut name = String::from("_entry");
                name.push_str(&i.to_string());
                append_to_pathbuf(mn, &name, "json")
            }).collect(),
        }
    }
    pub fn break_down(self) -> 
        (FnvHashMap<u32,Entry>, slog::Logger, HashMap<String,u32>)
    {
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
    pub fn export(&self, dst: PathBuf) -> Result<(), io::Error> {
        // write output to line-delimited JSON and CSV types
        let manifest = self.manifest(&dst);
        //write manifest
        let mut f = File::create(dst)?;
        let mn_s = serde_json::to_string(&manifest).unwrap();
        f.write_all(&mn_s.into_bytes())?;
        
        println!("Manifest: `{:?}`", manifest);

        //write addrs to csv
        let mut csv_w = csv::Writer::from_file(manifest.titles).unwrap();
        for (ref title, id) in &self.state.titles { 
            csv_w.encode((id,title)).unwrap(); 
        }

        //write entries to `self.threads` different files
        for (i,p) in manifest.entries.iter().enumerate() {
            println!("Writing to `{:?}`", p);
            let mut f = File::create(p)?;
            let dump = &self.state.dumps[i];
            for i in dump {
                let mut serial = serde_json::to_string(i).unwrap();
                serial.push('\n');
                f.write_all(&serial.into_bytes())?;
            }
        }
        Ok(()) 
    }

    /*
    pub fn import(src: PathBuf, log: slog::Logger) -> Result<Self,io::Error> { 
        assert!(src.is_file());
        let mut s = String::new();
        let mut f = File::open(src)?;
        f.read_to_string(&mut s).unwrap();
        let manifest: LinkManifest = serde_json::from_str(&s).unwrap();

        //populate titles
        let mut titles: HashMap<String,u32> = HashMap::with_capacity(manifest.size);
        let mut csv_r = csv::Reader::from_file(&manifest.titles)
            .unwrap().has_headers(false);
        for line in csv_r.decode() {
             let (id, title): (u32, String) = line.unwrap();
             titles.insert(title,id);
        }

        //populate entries
        let mut entries: Vec<Mutex<Vec<IndexedEntry>>> = Vec::with_capacity(manifest.threads);
        for i in 0..manifest.threads {
            let mut entries_v = Vec::with_capacity(manifest.size/manifest.threads);
            let f = File::open(&manifest.entries[i])?;
            let r = BufReader::new(f);
            for line in r.lines() {
                let e: IndexedEntry = serde_json::from_str(&line?).unwrap();
                entries_v.push(e);
            }
            entries.push(Mutex::new(entries_v));
        }

        Ok(LinkState {
            threads: manifest.threads,
            size:    manifest.size,
            log:     log,
            state:      LinkData {
                dumps: entries,
                titles: titles,
            }
        })
    }
    */

    pub fn import(src: PathBuf, log: slog::Logger) -> Result<Self,io::Error> { 
        assert!(src.is_file());
        let mut s = String::new();
        File::open(src).and_then(|mut f: File| f.read_to_string(&mut s))?;
        let manifest: LinkManifest = serde_json::from_str(&s).unwrap();

        //populate titles
        let mut titles: HashMap<String,u32> = HashMap::with_capacity(manifest.size);
        let mut csv_r = csv::Reader::from_file(&manifest.titles)
            .unwrap().has_headers(false);
        for line in csv_r.decode() {
             let (id, title): (u32, String) = line.unwrap();
             titles.insert(title,id);
        }

        let threads = manifest.entries.into_iter().map(|p| {
            thread::spawn(move || {
                let f = File::open(p)?;
                let r = BufReader::new(f);
                let err = "Deserializing data";
                r.lines().map(|l| l.map(|s| serde_json::from_str(&s).expect(err)))
                    .collect::<io::Result<Vec<IndexedEntry>>>()
            })
        }).collect::<Vec<thread::JoinHandle<io::Result<Vec<IndexedEntry>>>>>();
        // handle these unwraps better? look into `failure`?
        let data: Vec<Vec<IndexedEntry>> = threads.into_iter()
            .map(|t| t.join().unwrap().unwrap())
            .collect();

        Ok(LinkState {
            threads: manifest.threads,
            size:    manifest.size,
            log:     log,
            state:      LinkData {
                dumps: data,
                titles: titles,
            }
        })
    }
}

impl LinkData {
    //pub fn consolidate_links(links: Vec<Mutex<Vec<IndexedEntry>>>, size: usize) 
    pub fn consolidate_links(links: Vec<Vec<IndexedEntry>>, size: usize) 
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
