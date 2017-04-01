use csv;
use fnv::FnvHashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::cmp::Ordering;
use std::{f64, u64};

use super::{LinkState, LinkData, ProcData};
use super::Entry;
use super::link_data::IndexedEntry;

mod pagerank;

//const PRETTY_RANK_DUMPS: bool = true;

/*
// Store table metadata to be used at this step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdManifest {
    ranks:  Option<PathBuf>,
    titles: Option<PathBuf>,
}
*/


impl From<LinkState<LinkData>> for LinkState<ProcData> {
    fn from(old: LinkState<LinkData>) -> LinkState<ProcData> {
        // move addrs and entries from LinkData and compute pageranks
        
        // single threaded population
        let links = Self::consolidate_links(old.state.dumps, old.size);

        LinkState {
            threads:    old.threads,
            size:       old.size,
            log:        old.log,
            state:      ProcData {
                //links:  links,
                //ranks: r.into_iter().collect(),
                //ranks:  None,
                //titles: Self::build_title_table(&links),
                titles: old.state.titles,
                links:  links,
            }
        }
    }
}

impl LinkState<ProcData> {
    /*
    pub fn import(&mut self, src: &Path) {
        assert!(src.is_file());
        let mut s = String::new();
        let mut f = File::open(src).unwrap();
        f.read_to_string(&mut s).unwrap();
        let manifest: MdManifest = serde_json::from_str(&s).unwrap();
        // ranks will ONLY be computed iff we call self.build_ranks
        if let Some(p) = manifest.ranks {
            //self.state.ranks = Some(Self::import_ranks(&p));
        }
        if let Some(p) = manifest.titles {
            self.state.titles = Self::import_titles(&p);
        } else {
            // ALWAYS compute titles if they weren't given
            //self.state.titles = Some(Self::build_title_table(&self.state.links));
            self.build_title_table();
        }
    }
    
    fn import_titles(src: &Path) -> HashMap<String, u32> {
        let mut titles = HashMap::new();

        let mut csv_r = csv::Reader::from_file(src)
            .unwrap().has_headers(false);
        for line in csv_r.decode() {
            let (title, id): (String, u32) = line.unwrap();
            titles.insert(title, id);
        }
        titles
    }

    fn import_ranks(src: &Path) -> Vec<(u32,f64)> {
        let mut ranks: Vec<(u32, f64)> = Vec::new();

        let mut csv_r = csv::Reader::from_file(src)
            .unwrap().has_headers(false);
        for line in csv_r.decode() {
            // can't make parse conditioned on pretty-ness or type inference cries
            let (rank, id, _): (f64, u32, String) = line.unwrap();
            ranks.push((id, rank));
        }

        ranks
    }

    //fn build_title_table(links: &FnvHashMap<u32,Entry>) -> HashMap<String,u32> {
    pub fn build_title_table(&mut self) {
        unimplemented!();
        /*
        let links = &self.state.links;
        let mut titles = HashMap::with_capacity(links.len());
        let mut chopping_block = HashSet::new();

        for (&id, &Entry{ title: ref ti, .. }) in links.iter() {
            titles.insert(ti.to_owned(), TitleLookup::Orig(id));
            let caps = ti.to_uppercase();
            match titles.get(&caps) {
                Some(&TitleLookup::Orig(_)) => {},
                Some(&TitleLookup::Caps(x)) => {
                    if x != id {
                        chopping_block.insert(caps);
                    }
                },
                None => {
                    titles.insert(caps, TitleLookup::Caps(id));
                },
            }
        }

        for del in &chopping_block {
            titles.remove(del);
        }

        self.state.titles = titles;
        */
    }
    */
    /*
    pub fn build_title_table_(&mut self) {
        let links = &self.state.links;
        let mut originals: HashMap<String,u32> = HashMap::with_capacity(links.len());
        let mut capitals : HashMap<String,u8> = HashMap::with_capacity(links.len());

        // populate 'originals'
        for (&id, &Entry{ title: ref ti, .. }) in links.iter() {
            assert!(originals.contains_key(ti) == false);
            originals.insert(ti.to_owned(), id);
        }

        // populate 'capitals'
        for orig_ti in originals.keys() {
            let caps = orig_ti.to_uppercase();
            let e = capitals.entry(caps).or_insert(0);
            *e += 1;
        }

        // populate 'titles' with origs and capitals that don't interfere w/ origs
        let mut titles = originals.clone();
        for (orig_ti, &id) in &originals {
            let cap_ti = orig_ti.to_uppercase();
            let count = capitals.get(&cap_ti).unwrap();
            if *count == 1 && originals.contains_key(&cap_ti) == false {
                titles.insert(cap_ti, id);
            }
        }

        self.state.titles = Some(titles)
    }
    */

        // TODO: write data
    pub fn compute_ranks(&mut self, path: &PathBuf) -> Result<(), csv::Error> {
        let pr_log = self.log.new(o!(
                "damping" => pagerank::DAMPING_FACTOR,
                "epsilon" => pagerank::MAX_ERROR));
        let r = pagerank::Graph::new(&self.state.links).get_ranks(pr_log);
        // sort floats; will all be less than 
        // so should be the same as sorting by the negative reciprocal
        let mut sorted_r: Vec<_> = r.into_iter().collect();
        sorted_r.sort_by_key(|&(_,r)| {
            assert!(r.is_normal());
            assert!(r.is_sign_positive());
            assert!(r <= 1.0);
            u64::MAX - r.recip() as u64
        });
        let mut csv_w = csv::Writer::from_file(path)?;
        for (id,rank) in sorted_r {
            let ref title = self.state.links.get(&id).unwrap().title;
            csv_w.encode((rank,id,title))?;
        }
        Ok(())
    }

    /*
    fn build_ranks(old: LinkState<LinkData>, output: &Path) -> Self {
        let mut this: LinkState<ProcData> = old.into();
        // compute ranks
        let pr_log = this.log.new(o!(
                "damping" => pagerank::DAMPING_FACTOR,
                "epsilon" => pagerank::MAX_ERROR));
        let r = pagerank::Graph::new(&this.state.links).get_ranks(pr_log);
        this.state.ranks = Some(r.into_iter().collect());
        // save ranks
        this.export(&PathBuf::from(output), PRETTY_RANK_DUMPS).unwrap();

        this
    }
    */
    fn consolidate_links(links: Vec<Mutex<Vec<IndexedEntry>>>, size: usize) 
        -> FnvHashMap<u32,Entry> 
    {
        let mut hm: FnvHashMap<u32,Entry> = 
            FnvHashMap::with_capacity_and_hasher(size, Default::default());
        for bucket in links {
            for ie in bucket.into_inner().unwrap() {
                let (id, entry) = ie.decompose();
                hm.insert(id, entry);
            }
        }
        hm
    }
    /*
    fn from_ranks(old: LinkState<LinkData>, ranks_path: &Path) -> Self {
        let links = Self::consolidate_links(old.state.dumps, old.size);
        //populate ranks from csv file
        let mut ranks: Vec<(u32,f64)> = Vec::with_capacity(old.size);

        let mut csv_r = csv::Reader::from_file(ranks_path)
            .unwrap().has_headers(false);
        for line in csv_r.decode() {
            let (id, rank): (u32, f64) = line.unwrap();
            ranks.push((id, rank));
        }

        LinkState {
            threads: old.threads,
            size:    old.size,
            log:     old.log,
            state:   ProcData {
                titles: Self::build_title_table(&links),
                links: links,
                ranks: Some(ranks.into_iter().collect()),
            }
        }
    }
    */
    /*
    fn save_ranks(&self, ranks: &Vec<(u32,f64)>, ranks_path: &Path) -> Result<(),csv::Error> {
        // note about writing/reading files: they won't be identical
        // because we're iterating through a hashmap
        // but they are identical content-wise
        let mut csv_w = csv::Writer::from_file(ranks_path)?;
        for datum in ranks {
            csv_w.encode(datum)?;
        }
        Ok(())
    }
    */
    fn pretty_ranks(&self, ranks: &[(u32,f64)], ranks_path: &Path) -> Result<(),csv::Error> {
        //sort greatest-to-least
        // (RANK, ID, TITLE)
        let mut sorted_ranks = ranks.to_vec();
        sorted_ranks.sort_by(|&(a_i,a_r),&(b_i,b_r)| {
            //sort by floats, which Ord does not provide
            assert!(!a_r.is_nan(), "Page {} had a NaN rank", a_i);
            assert!(!b_r.is_nan(), "Page {} had a NaN rank", b_i);
            //match (a_r > b_r, a_r == b_r) {
            match (a_r > b_r, (a_r - b_r).abs() < f64::EPSILON) {
                (true, _) => Ordering::Less,
                (_, true) => Ordering::Equal,
                _         => Ordering::Greater,
            }
        });
        
        // write using interesting csv data
        let mut csv_w = csv::Writer::from_file(ranks_path)?;
        for (id,rank) in sorted_ranks {
            let title = &self.state.links[&id].title;
            csv_w.encode((rank,id,title))?;
        }
        Ok(())
    }
        /*
    pub fn export(&mut self, path: &PathBuf) -> Result<(),csv::Error> {
        let manifest = MdManifest {
            titles: Some(append_to_pathbuf(path, "_titles", "csv")),
            /*ranks: if self.state.ranks.is_some() {
                Some(append_to_pathbuf(path, "_ranks", "csv"))
            } else {
                None
            }*/
        };
        let mut f = File::create(path).unwrap();
        let mn_s = serde_json::to_string(&manifest).unwrap();
        f.write_all(&mn_s.into_bytes()).unwrap();

        // save ranks and titles
        /*
        if self.state.titles.is_none() {
            self.build_title_table();
        }
        if let Some(ref titles) = self.state.titles {
            let mut csv_w = csv::Writer::from_file(manifest.titles.unwrap())?;
            for (title,id) in titles {
                let lookup = serde_json::to_string(id).unwrap();
                csv_w.encode((title,lookup))?;
            }
        } else {
            panic!("No titles to export!");
        }

        if let Some(ref ranks) = self.state.ranks {
            let ranks_fn = Self::pretty_ranks;
            //let ranks_fn = if pretty {
            //    Self::pretty_ranks
            //} else {
            //    Self::pretty_ranks
            //};
            // lol is this gonnna work?
            ranks_fn(self, ranks, &manifest.ranks.unwrap())?;
        }
        */
        Ok(())
    }
*/
    /*
    fn data(&self) {
        info!(self.log, "State of ProcData:");
        info!(self.log, "Number of entries: {}", self.state.links.len());
        if let Some(ref r) = self.state.ranks {
            info!(self.log, "Number of ranks: {}", r.len());
        } else {
            info!(self.log, "No rank data");
        }

        if let Some(ref r) = self.state.ranks {
            let rank_sum: f64 = r.iter().map(|&(_,r)| r).sum();
            let total_links: usize = self.state.links
                .iter()
                .map(
                    |(_, &Entry{ children: ref c, parents: ref p, .. })| 
                    c.len() + p.len())
                .sum();
            info!(self.log, "Sum of all ranks:   {}", rank_sum);
            info!(self.log, "Number of links:    {}", total_links);
        }
    }
    */
}