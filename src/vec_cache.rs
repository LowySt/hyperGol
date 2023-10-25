use std::hash::Hash;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use bytebuffer::ByteBuffer;
use byte_slice_cast::*;

pub struct CachedIndex<T>
{
    hash: u64,
    cursor: T,
}

pub struct VectorCache
{
    pub strings:     Vec::<CachedIndex::<u32>>,
    pub numbers:     Vec::<CachedIndex::<u16>>,
    pub names:       Vec::<CachedIndex::<u16>>, //TODO We are gonna surpass 16bits veery soon!
    pub gs:          Vec::<CachedIndex::<u16>>,
    pub pe:          Vec::<CachedIndex::<u16>>,
    pub alignment:   Vec::<CachedIndex::<u16>>,
    pub types:       Vec::<CachedIndex::<u16>>,
    pub subtypes:    Vec::<CachedIndex::<u16>>,
    pub archetypes:  Vec::<CachedIndex::<u16>>,
    pub sizes:       Vec::<CachedIndex::<u16>>,
    pub senses:      Vec::<CachedIndex::<u16>>,
    pub auras:       Vec::<CachedIndex::<u16>>,
    pub immunities:  Vec::<CachedIndex::<u16>>,
    pub resistances: Vec::<CachedIndex::<u16>>,
    pub weaknesses:  Vec::<CachedIndex::<u16>>,
    pub special:     Vec::<CachedIndex::<u16>>,
    pub spells:      Vec::<CachedIndex::<u32>>,
    pub talents:     Vec::<CachedIndex::<u16>>,
    pub skills:      Vec::<CachedIndex::<u32>>,
    pub languages:   Vec::<CachedIndex::<u16>>,
    pub specials:    Vec::<CachedIndex::<u32>>,
    pub environment: Vec::<CachedIndex::<u16>>,
}

impl VectorCache
{
    pub fn new(pre_alloc: usize) -> VectorCache
    {
        let result = VectorCache {
            strings:     Vec::<CachedIndex::<u32>>::with_capacity(pre_alloc),
            numbers:     Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            names:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            gs:          Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            pe:          Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            alignment:   Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            types:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            subtypes:    Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            archetypes:  Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            sizes:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            senses:      Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            auras:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            immunities:  Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            resistances: Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            weaknesses:  Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            special:     Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            spells:      Vec::<CachedIndex::<u32>>::with_capacity(pre_alloc),
            talents:     Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            skills:      Vec::<CachedIndex::<u32>>::with_capacity(pre_alloc),
            languages:   Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            specials:    Vec::<CachedIndex::<u32>>::with_capacity(pre_alloc),
            environment: Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
        };
        return result;
    }
}

pub fn add_entry_if_missing_u32(cache: &mut Vec<CachedIndex<u32>>, buf: &mut ByteBuffer, entry_data_str: &str) -> u32
{
    let replace_shit = entry_data_str.replace("\u{2012}", "-");
    let replace_shit = replace_shit.replace("\u{200b}", "");
    let replace_shit = replace_shit.replace("\u{2011}", "-");
    let replace_shit = replace_shit.replace("\u{2013}", "-");
    let replace_shit = replace_shit.replace("\u{2014}", "-");
    let replace_shit = replace_shit.replace("\u{2018}", "'");
    let replace_shit = replace_shit.replace("\u{2019}", "'");
    let replace_shit = replace_shit.replace("\u{201c}", "\"");
    let replace_shit = replace_shit.replace("\u{201d}", "\"");
    let replace_shit = replace_shit.replace("\u{2026}", "...");
    let replace_shit = replace_shit.replace("\u{2212}", "-");
    let replace_shit = replace_shit.replace("\u{2800}", "");
    let replace_shit = replace_shit.replace("\u{fb01}", "fi");
    let replace_shit = replace_shit.replace("\u{fb02}", "fl");
    let entry_data   = replace_shit.as_bytes();
    let entry_len    = replace_shit.len();
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u32; }
    
    let mut hasher = DefaultHasher::new();
    entry_data.hash(&mut hasher);
    let hash = hasher.finish();
    
    for idx in 0..cache.len()
    { 
        if cache[idx].hash == hash { return cache[idx].cursor; }
    }
    
    let mut cursor: usize = 0;
    cursor = buf.get_wpos();
    
    buf.write_bytes([entry_len as u16].as_byte_slice());
    buf.write_bytes(entry_data);
    
    let write_cursor = buf.get_wpos();
    let new_buff_size = (buf.len() - 4) as u32;
    buf.set_wpos(0);
    buf.write_bytes([new_buff_size].as_byte_slice());
    buf.set_wpos(write_cursor);
    
    let mut cached_index = CachedIndex { hash: hash, cursor: cursor as u32 };
    cache.push(cached_index);
    
    return cursor as u32;
}

pub fn add_entry_if_missing(cache: &mut Vec<CachedIndex<u16>>, buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let replace_shit = str::replace(entry_data_str, "\u{2012}", "-");
    let replace_shit = replace_shit.replace("\u{200b}", "");
    let replace_shit = replace_shit.replace("\u{2011}", "-");
    let replace_shit = replace_shit.replace("\u{2013}", "-");
    let replace_shit = replace_shit.replace("\u{2014}", "-");
    let replace_shit = replace_shit.replace("\u{2018}", "'");
    let replace_shit = replace_shit.replace("\u{2019}", "'");
    let replace_shit = replace_shit.replace("\u{201c}", "\"");
    let replace_shit = replace_shit.replace("\u{201d}", "\"");
    let replace_shit = replace_shit.replace("\u{2026}", "...");
    let replace_shit = replace_shit.replace("\u{2212}", "-");
    let replace_shit = replace_shit.replace("\u{2800}", "");
    let replace_shit = replace_shit.replace("\u{fb01}", "fi");
    let replace_shit = replace_shit.replace("\u{fb02}", "fl");
    let entry_data   = replace_shit.as_bytes();
    let entry_len    = replace_shit.len();
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u16; }
    
    let mut hasher = DefaultHasher::new();
    entry_data.hash(&mut hasher);
    let hash = hasher.finish();
    
    for idx in 0..cache.len()
    { 
        if cache[idx].hash == hash { return cache[idx].cursor; }
    }
    
    let mut cursor: usize = 0;
    cursor = buf.get_wpos();
    
    buf.write_bytes([entry_len as u16].as_byte_slice());
    buf.write_bytes(entry_data);
    
    let write_cursor = buf.get_wpos();
    let new_buff_size = (buf.len() - 4) as u32;
    buf.set_wpos(0);
    buf.write_bytes([new_buff_size].as_byte_slice());
    buf.set_wpos(write_cursor);
    
    let mut cached_index = CachedIndex { hash: hash, cursor: cursor as u16 };
    cache.push(cached_index);
    
    return cursor as u16;
}

pub fn add_entry_if_missing_dbg(cache: &mut Vec<CachedIndex<u16>>, buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let replace_shit = str::replace(entry_data_str, "\u{2012}", "-");
    let replace_shit = replace_shit.replace("\u{200b}", "");
    let replace_shit = replace_shit.replace("\u{2011}", "-");
    let replace_shit = replace_shit.replace("\u{2013}", "-");
    let replace_shit = replace_shit.replace("\u{2014}", "-");
    let replace_shit = replace_shit.replace("\u{2018}", "'");
    let replace_shit = replace_shit.replace("\u{2019}", "'");
    let replace_shit = replace_shit.replace("\u{201c}", "\"");
    let replace_shit = replace_shit.replace("\u{201d}", "\"");
    let replace_shit = replace_shit.replace("\u{2026}", "...");
    let replace_shit = replace_shit.replace("\u{2212}", "-");
    let replace_shit = replace_shit.replace("\u{2800}", "");
    let replace_shit = replace_shit.replace("\u{fb01}", "fi");
    let replace_shit = replace_shit.replace("\u{fb02}", "fl");
    let entry_data   = replace_shit.as_bytes();
    let entry_len    = replace_shit.len();
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u16; }
    
    let mut hasher = DefaultHasher::new();
    entry_data.hash(&mut hasher);
    let hash = hasher.finish();
    
    for idx in 0..cache.len()
    { 
        if cache[idx].hash == hash { return cache[idx].cursor; }
    }
    
    println!("{}", entry_data_str);
    
    let mut cursor: usize = 0;
    cursor = buf.get_wpos();
    
    buf.write_bytes([entry_len as u16].as_byte_slice());
    buf.write_bytes(entry_data);
    
    let write_cursor = buf.get_wpos();
    let new_buff_size = (buf.len() - 4) as u32;
    buf.set_wpos(0);
    buf.write_bytes([new_buff_size].as_byte_slice());
    buf.set_wpos(write_cursor);
    
    let mut cached_index = CachedIndex { hash: hash, cursor: cursor as u16 };
    cache.push(cached_index);
    
    return cursor as u16;
}
