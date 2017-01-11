use diesel;

// Database Layout: One row composed of:
//  Cache:
//      Path:       array of 32-bit unsigned integers indicating the path, including src/dst
//      Timestamp:  the last time this entry was accessed
//      Count:      the number of times this entry was accessed
//  Addresses:
//      Title:      titles (that we can fuzzy-select?) (`tsvector`?)
//      Address:    u32 page_id
//
