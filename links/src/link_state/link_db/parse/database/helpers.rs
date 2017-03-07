
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
