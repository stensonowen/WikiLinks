use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{self, Read, /*Write*/};
use std::path::PathBuf;
use std::fs::File;
use serde_json;
use slog;
use csv;

use super::{LinkState, LinkDb, LinkData};
use super::Entry;


impl From<LinkState<LinkDb>> for LinkState<LinkData> {
    fn from(old: LinkState<LinkDb>) -> LinkState<LinkData> {
        // entries will become into lookup table
        // addresses and ranks feed into PostgreSQL
        
        let (entries_i, addrs_i) = old.state.parts();
        let mut entries: Vec<Mutex<Vec<(u32,Entry)>>> = Vec::with_capacity(old.threads);
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
    fn to_file(&self, mn: PathBuf) -> Result<(), io::Error> {
        let manifest = Manifest {
            threads: self.threads,
            size:    self.size,
            addrs:   mn.clone().with_extension("addr").with_extension("csv"),
            entries: (0..self.threads)
                .map(|i| mn.clone()
                     .with_extension("entry")
                     .with_extension(i.to_string())
                     .with_extension("json"))
                .collect()
        };
        //let addr_arc = Arc::new(&self.state.addrs);
        let addr_arc = Arc::new(self.state.addrs.clone());
        let addrs = addr_arc.clone();
        let addr_thread = thread::spawn(move || {
            let mut writer = csv::Writer::from_file(manifest.addrs).unwrap();
            for a in addrs.iter() { 
                //how to not copy?
                writer.encode(a).unwrap(); 
            }
        });

        let _entry_threads: Vec<_> = (0..self.threads).map(|_i| {
            //thread::spawn(move || {
                //let mut f = File::open(manifest.entries[i]).unwrap();
            //})
        }).collect();

        addr_thread.join().unwrap();
        Ok(()) 
    }

    fn from_file(src: PathBuf, log: slog::Logger) -> Self { 
        assert!(src.is_file());
        let mut s = String::new();
        let mut f = File::open(src).unwrap();
        f.read_to_string(&mut s).unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();

        LinkState {
            threads: manifest.threads,
            size:    manifest.size,
            log:     log,
            state:      LinkData {
                dumps: vec![],
                addrs: vec![],
            }
        }
    }

}

