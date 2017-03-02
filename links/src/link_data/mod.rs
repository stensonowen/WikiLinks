use std::sync::{/*Arc,*/ Mutex};
//use std::thread;
use std::io::{self, Read, Write, BufRead, BufReader};
use std::path::PathBuf;
use std::fs::File;
use std::ffi::OsString;
use serde_json;
use slog;
use csv;

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
struct Manifest {
    threads: usize,
    size:    usize,
    entries: Vec<PathBuf>,
    addrs: PathBuf,
}



impl LinkState<LinkData> {
    // need to read from or write to files to restore from/to this state
    pub fn to_file(&self, mn: PathBuf) -> Result<(), io::Error> {
        // write output to line-delimited JSON and CSV types
        let mn_ = mn.clone();
        let mn_fn = mn_.file_name().unwrap();
        let manifest = Manifest {
            threads: self.threads,
            size:    self.size,
            addrs:   {
                let mut name = OsString::from(mn_fn.clone());
                name.push(OsString::from("_addr"));
                mn_.with_file_name(name).with_extension("csv")
            },
            entries: (0..self.threads)
                .map(|i| {
                    let mut name = OsString::from(mn_fn.clone());
                    name.push(OsString::from("_entry"));
                    name.push(i.to_string());
                    mn.clone().with_file_name(name).with_extension("json")
                })
                .collect()
        };
        //write manifest
        let mut f = File::create(mn)?;
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

    fn from_file(src: PathBuf, log: slog::Logger) -> Result<Self,io::Error> { 
        assert!(src.is_file());
        let mut s = String::new();
        let mut f = File::open(src)?;
        f.read_to_string(&mut s).unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();

        //populate addresses
        let mut addrs: Vec<(String,u32)> = Vec::with_capacity(manifest.size);
        let mut csv_r = csv::Reader::from_file(&manifest.addrs).unwrap();
        for line in csv_r.decode() {
             let (id, title): (u32, String) = line.unwrap();
             addrs.push((title,id));
        }

        //populate entries
        let mut entries: Vec<Mutex<Vec<IndexedEntry>>> = Vec::with_capacity(manifest.threads);
        for i in 0..manifest.threads {
            let mut entries_v = Vec::with_capacity(manifest.size/manifest.threads);
            //entries.push(Mutex::new(Vec::with_capacity(manifest.size/manifest.threads)));
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

