use std::collections::{HashSet, HashMap};
use super::{Path, PathError, LinkState, HashLinks};
use std::mem::swap;

const MAX_DEPTH: usize = 10;

// Find the shortest path between articles



enum SearchDirection { Up, Down, }

impl LinkState<HashLinks> {
    pub fn print_bfs(&self, src: u32, dst: u32) {
        let bfs = self.bfs(src, dst);
        bfs.print(&self.state.links);
    }
    fn bfs(&self, src: u32, dst: u32) -> Path {
        use self::SearchDirection::*;
        // perform breadth-first-search using entries from src to dst
        // should this be multithreaded??
        // TODO: investigate using a Bloom Filter for checking intersection quickly
        //  Might be faster: that is its thing
        //  Might be negligible: uses hashing and a lookup table, which is basically a hashmap
        //  Might be useful if this were multithreaded?
        // TODO: play with different approximate hints
        //  i.e. starting/changing hashmap capacity via guessing
        // TODO: fix code duplication (top-down and bottom-up doing almost the same thing
        // TODO: there's a fair bit of redundancy: top_down_n ⊆ src_seen ⊇ top_down_o

        if src == dst {
            return Path{ src: src, dst: dst, path: Ok(vec![src])};
        }

        //`seen` pages: the id we've encountered and the id that linked to it (parent OR child)
        let mut src_seen: HashMap<u32, u32> = HashMap::new();
        let mut dst_seen: HashMap<u32, u32> = HashMap::new();

        // keep track of articles we're checking in each direction
        // each direction should have an immutable bank and a mutable collector (that swap)
        // start from src, follow children
        let mut top_down: HashSet<u32> = HashSet::new();
        top_down.insert(src);
        // start from dst, follow parents
        let mut bottom_up: HashSet<u32> = HashSet::new();
        bottom_up.insert(dst);

        src_seen.insert(src, src);
        dst_seen.insert(dst, dst);

        let mut tmp: HashSet<u32> = HashSet::new();

        for _ in 0..MAX_DEPTH {
            if let Some(midpoint) = self.iterate(&top_down, &mut tmp, Down,
                                            &mut src_seen, &dst_seen) {
                return Path { src: src, dst: dst,
                    path: Ok(extract_path(src, dst, &src_seen, &dst_seen, midpoint)) };
            }
            top_down.clear();
            swap(&mut top_down, &mut tmp);

            if let Some(midpoint) = self.iterate(&bottom_up, &mut tmp, Up,
                                            &mut dst_seen, &src_seen) {
                return Path { src: src, dst: dst,
                    path: Ok(extract_path(src, dst, &src_seen, &dst_seen, midpoint)) };
            }
            bottom_up.clear();
            swap(&mut bottom_up, &mut tmp);

            if bottom_up.is_empty() || top_down.is_empty() {
                return Path { src: src, dst: dst, path: Err(PathError::NoSuchPath) };
            }
        }
        Path { src: src, dst: dst, path: Err(PathError::Terminated(MAX_DEPTH)) }
    }

    fn iterate(&self,
               prev_line: &HashSet<u32>,
               new_line: &mut HashSet<u32>,
               direction: SearchDirection,
               reachable: &mut HashMap<u32, u32>,
               targets: &HashMap<u32, u32>)
               -> Option<u32> 
    {
        // go through every element in `prev_line`, adding parents/children to `next_line`
        // as we see each entry, add it to `reachable`.
        // If we find an element in `targets` and `reachable`, searching is over: return it
        use self::SearchDirection::*;
        for &i in prev_line {
            // link_start
            let pool = match direction {
                Up => &self.state.links.get(&i).unwrap().parents,
                Down => &self.state.links.get(&i).unwrap().children,
            };
            for &j in pool {
                // link_end
                // for each link_end, if unseen, mark it as reachable
                if *reachable.entry(j).or_insert(i) == i {
                    // we successfully inserted `seen[j]=i`, and it wasn't there before
                    if targets.contains_key(&j) {
                        // we found an element that is in both `reachable` and `target`
                        return Some(j);
                    }
                    new_line.insert(j);
                }
            }
        }
        None
    }
}
fn extract_path(src: u32,
                dst: u32,
                src_seen: &HashMap<u32, u32>,
                dst_seen: &HashMap<u32, u32>,
                common: u32)
                -> Vec<u32> {
    // given entries findable from the src, those from the dst, and an intersecting entry,
    // find the path from src to dst
    // For now, the path should include the src and dst ids
    let mut path = vec![common];
    // first find the path from the midpoint to the src (will be backwards)
    let mut current = common;
    while current != src {
        // could just iterate until it finds something without a parent? no passing src/dst ?
        // this is easier to debug for now
        current = *src_seen.get(&current).unwrap();
        path.push(current);
    }
    path.reverse();
    current = common;
    while current != dst {
        current = *dst_seen.get(&current).unwrap();
        path.push(current);
    }
    path
}

/*
pub fn print_path(res: Result<Vec<u32>, Error>, url: Option<&str>) {
    match res {
        Ok(path) => {
            println!("Found a path with {} steps: ", path.len() - 1);
            for id in &path {
                let title = ENTRIES.get(id).unwrap().title;
                if let Some(prefix) = url {
                    println!("\t{: >10}: \t{: <16} (https://{}.wikipedia.org/?curid={: <10})",
                             id,
                             title,
                             prefix,
                             id);
                } else {
                    println!("\t{: >12}: \t`{}`", id, title);
                }
            }
        },
        Err(Error::Terminated(t)) => println!("The search was terminated after {} rounds", t),
        Err(Error::NoSuchPath) => println!("No such path of any length exists"), 
    }
}

pub fn format_path(res: Result<Vec<u32>, Error>, lang: &str) -> String {
    //output html
    match res {
        Ok(path) => {
            let mut s = String::new();
            s.push_str(r"<body>");
            for id in &path {
                let title = ENTRIES.get(id).unwrap().title;
                s.push_str(
                    &format!(r#"<p><a href=\"https://{}.wikipedia.org/?curid={}\">`{}`</a></p>"#,
                             lang, id, title));
            }
            s.push_str(r"</body>");
            s
        },
        Err(Error::Terminated(t)) => format!(r"The search was terminated after {} rounds", t),
        Err(Error::NoSuchPath) => String::from(r"No such path of any length exists"),
    }
}

pub fn annotate_path(res: Vec<u32>, lang: &str) -> Vec<(&'static str, String)> {
    res.iter().map(|id| (
            ENTRIES.get(id).unwrap().title,
            format!("https://{}.wikipedia.org/?curid={}", lang, id)
            ))
        .collect()
}
*/
