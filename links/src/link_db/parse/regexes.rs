
//const IS_SIMPLE: bool = true;   //small parsing differences between simple and English wikis
use super::super::super::IS_SIMPLE; //a little awkward, but this is harder to forget

// NOTE about namespaces:
//  Namespaces complicate things. The 0 namespace (i.e. Main, "real" articles) is the
//  overwhelmingly most relevant one; others represent User pages, help pages, etc.
//  Using namespaces poses two major difficulties:
//  1. They double the length of an address (they're only i16s but they need padding)
//  2. They introduce ambiguity because two articles in different namespaces can share a name.
//  These are not insurmountable challenges, but I'm not sure they're worth solving.
//

pub fn pagelinks_regex() -> String {
    // String::from(r"\((\d)+,0,'([^'\\]*(?:\\.[^'\\]*)*)',-?\d+\)")
    String::from(r"\((\d+),0,'([^'\\]*(?:\\.[^'\\]*)*)',0\)")
}

pub fn redirect_regex() -> String {
    // matches all 9278254 english wiki entries
    // matches all   58130  simple wiki entries
    let page_id = r"(\d+)";
    let page_nmsp = r"0";
    let page_title = r"'([^'\\]*(?:\\.[^'\\]*)*)'";
    let page_iw = r"(?:'.*?'|NULL)";  //can be but never has been NULL (slowdown: ~30%)
    let page_frag = r"(?:'.*?'|NULL)";
    let re_body = vec![page_id, page_nmsp, page_title, page_iw, page_frag];
    format!(r"\({}\)", re_body.join(","))
}


pub fn pages_regex() -> String {
    // we make a few assumptions here; matches everything in the english page.sql dump
    let page_id = r"(\d+)";     //captured, positive non-null number
    let page_nmsp = r"0";   //captured; should never be negative(?) (0-15 ∪ 1000-2**31)
    let page_title = r"'([^'\\]*(?:\\.[^'\\]*)*)'"; //surrounded by `'`s, which can be escaped
    let page_restrs = r"'.*?'"; 	//not always empty, but never has escaped quotes
    let page_counter = r"\d+";       //non-captured positive number; count will be wrong
    let page_is_redr = r"(0|1)";     // 0 or 1, to indicate binary value; capture? usefulness?
    let page_is_new = r"(?:0|1)";   // 0 or 1; info is useless to us
    let page_random = r"[\d\.]+";   //random(?) unsigned double
    let page_tchd = r"'\d+'";     //timestamp; irrelevant; cannot contain `\'`s (?)
    let page_ln_upd = r"(?:'\d+'|NULL)"; //another irrelevant timestamp, but it can be null
    let page_latest = r"\d+";       //unsigned int: latest revision number
    let page_len = r"\d+";       //unsigned int: page len (or weird value)
    let page_ntc = r"(?:0|1)";   //what is this? it's only in the Simple wiki dump
    let page_cont_md = r"(?:'.*?'|NULL)"; //probably never contains escaped quotes?
    let page_lang = r"NULL";  	//think it'll always be null?

    let mut re_body = vec![page_id,
                           page_nmsp,
                           page_title,
                           page_restrs,
                           page_counter,
                           page_is_redr,
                           page_is_new,
                           page_random,
                           page_tchd,
                           page_ln_upd,
                           page_latest,
                           page_len,
                           // page_ntc,
                           page_cont_md,
                           page_lang];
    if IS_SIMPLE {
        re_body.insert(12, page_ntc);
    }

    format!(r"\({}\)", re_body.join(","))
}
