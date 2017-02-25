
//pub type PAGE_ID = u32;

#[derive(Debug)]
pub enum Entry {
    // either a unique page and its link data
    Page {
        title: String,
        children: Vec<u32>,
        parents: Vec<u32>,
    },
    // or the redirect page and its address
    // these will eventually be taken out of db::entries
    Redirect {
        title: String,
        target: Option<u32>,
    }
}

//what phase the database is in 
//TODO should I get rid of the numbers? They don't matter except that without 
// them it might not be clear that the order of the values is what determines
// their inequality; here concrete values make it clearer
#[derive(PartialEq, PartialOrd, Debug)]
pub enum State {
    Begin           = 0,
    AddPages        = 1,
    AddRedirects    = 2,
    TidyEntries     = 3,
    AddLinks        = 4,
    Done            = 5,
}

impl Entry {
    //codegen: create valid `static` copies of data for codegen.rs
    //no reason to keep Entry::Redirect in the final table
    pub fn codegen_links(&self, addr: u32) -> Option<String> {
        if let &Entry::Page { children: ref c, parents: ref p, .. } = self {
            // e.g. `static CHILDREN_1111: [u32;3] = [5555,6666,7777];`
            let mut s = String::with_capacity(85);
            s.push_str(&format!("pub static PAGE_{}_C: [u32;{}] = {:?};\n", addr, c.len(), c));
            s.push_str(&format!("pub static PAGE_{}_P: [u32;{}] = {:?};\n", addr, p.len(), p));
            Some(s)
        } else {
            None
        }
    }
    pub fn codegen_page(&self, addr: u32) -> Option<String> {
        if let &Entry::Page { title: ref t, .. } = self {
            Some(format!("Page {{ title: {:?}, children: &PAGE_{}_C, parents: &PAGE_{}_P }}", 
                         t, addr, addr))
        } else {
            None
        }
    }
}
