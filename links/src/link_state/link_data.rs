
use {slog, csv, serde_json};
use std::sync::Mutex;
use std::io::{self, Read, Write, BufRead, BufReader};
use std::path::PathBuf;
use std::fs::File;
use std::ffi::OsString;

use super::{LinkState, LinkDb, LinkData};
use super::Entry;

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedEntry {
    id: u32,
    title: String,
    parents: Vec<u32>,
    children: Vec<u32>,
}

impl IndexedEntry {
    pub fn from(i: u32, t: String, p: Vec<u32>, c: Vec<u32>) -> Self {
        IndexedEntry {
            id: i,
            title: t,
            parents: p,
            children: c,
        }
    }
    pub fn decompose(self) -> (u32,Entry) {
        (self.id, Entry {
            title: self.title,
            parents: self.parents,
            children: self.children,
        })
    }
}

impl From<LinkState<LinkDb>> for LinkState<LinkData> {
    fn from(old: LinkState<LinkDb>) -> LinkState<LinkData> {
        // entries will become into lookup table
        // addresses and ranks feed into PostgreSQL
        
        let (entries_i, addrs_i) = old.state.parts();
        let mut entries: Vec<Mutex<Vec<IndexedEntry>>> = Vec::with_capacity(old.threads);
        let addrs: Vec<(String,u32)> = addrs_i.collect();

        //seems like there should be a more functional way to do this
        //  if .take() didn't consume?  doesn't shallow copy??
        // could stand to be refactored
        let size = old.size / old.threads + 1;
        for _ in 0..old.threads+1 {
            entries.push(Mutex::new(Vec::with_capacity(size)));
        }
        let mut count = 0usize;
        for entry in entries_i {
            entries[count/size].get_mut().unwrap().push(entry);
            count += 1;
        }
        assert_eq!(count, old.size, "Lost elements populating LinkDb");

        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      LinkData {
                dumps: entries,
                addrs: addrs,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkManifest {
    threads: usize,
    size:    usize,
    entries: Vec<PathBuf>,
    addrs:   PathBuf,
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
            addrs:      append_to_pathbuf(mn, "_addr", "csv"),
            entries:    (0..self.threads).map(|i| {
                let mut name = String::from("_entry");
                name.push_str(&i.to_string());
                append_to_pathbuf(mn, &name, "json")
            }).collect(),
        }
    }
    //pub fn to_file(&self, mn: PathBuf) -> Result<(), io::Error> {
    pub fn export(&self, dst: PathBuf) -> Result<(), io::Error> {
        // write output to line-delimited JSON and CSV types
        let manifest = self.manifest(&dst);
        //write manifest
        let mut f = File::create(dst)?;
        let mn_s = serde_json::to_string(&manifest).unwrap();
        f.write_all(&mn_s.into_bytes())?;
        
        println!("Manifest: `{:?}`", manifest);

        //write addrs to csv
        let mut csv_w = csv::Writer::from_file(manifest.addrs).unwrap();
        for &(ref title, id) in &self.state.addrs { 
            csv_w.encode((id,title)).unwrap(); 
        }

        //write entries to `self.threads` different files
        for (i,p) in manifest.entries.iter().enumerate() {
            println!("Writing to `{:?}`", p);
            let mut f = File::create(p)?;
            let dump = &self.state.dumps[i];
            for i in dump.lock().unwrap().iter() {
                let mut serial = serde_json::to_string(i).unwrap();
                serial.push('\n');
                f.write_all(&serial.into_bytes())?;
            }
        }

        Ok(()) 
    }

    //pub fn from_file(src: PathBuf, log: slog::Logger) -> Result<Self,io::Error> { 
    pub fn import(src: PathBuf, log: slog::Logger) -> Result<Self,io::Error> { 
        assert!(src.is_file());
        let mut s = String::new();
        let mut f = File::open(src)?;
        f.read_to_string(&mut s).unwrap();
        let manifest: LinkManifest = serde_json::from_str(&s).unwrap();

        //populate addresses
        let mut addrs: Vec<(String,u32)> = Vec::with_capacity(manifest.size);
        let mut csv_r = csv::Reader::from_file(&manifest.addrs)
            .unwrap().has_headers(false);
        for line in csv_r.decode() {
             let (id, title): (u32, String) = line.unwrap();
             addrs.push((title,id));
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
                addrs: addrs,
            }
        })
    }

}

