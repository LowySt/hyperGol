use bytebuffer::ByteBuffer;

use crate::entries::*;
use crate::vec_cache::*;

/*NOTE: The talents are mapped in a 32 bit value as follows
*       [0|1] [0|1] [0|1] [1 UNUSED] [16 bit interned talent string] [12 bit index in talent module]
*         ^     ^     ^                   ^                                  ^
 *         |     |     | Mithic            |                                  |
*         |    Bonus                      |                                  |
*         |                               |                                  |
   *        The high bit (bit index 31)     The 16 bits in indices 27..12      The low 12 bits in indices 11..0
 *         is used to indicate wether     indicate the interned display      indicate the index in the talent module
*         the display name has been      name idx in the talents buffer     to retrieve all informations about the
 *         interned or not.               if present.                        talent (The entire talent page).
*         (0 NO, 1 YES)
*/

const TALENT_INTERN_BIT_U32:   u32 = 0x80000000;
const TALENT_BONUS_BIT_U32:    u32 = 0x40000000;
const TALENT_MITHIC_BIT_U32:   u32 = 0x20000000;
const TALENT_INTERN_IDX_SHIFT: u32 = 12;
const TALENT_INTERN_MASK:      u32 = 0x0FFFF000;
const TALENT_MODULE_IDX_MASK:  u32 = 0x00000FFF;

const TALENT_NOT_FOUND:        u32 = 0x00000FFF;

fn find_in_talents(cache: &mut Vec::<CachedIndex::<u32>>, buf: &mut ByteBuffer, needle: &str, page_addr: &str) -> u32
{
    let hashed = get_cursor_u32(cache, needle.as_bytes());
    
    match hashed {
        (hash, 0u32) => {
            //NOTE: Seek to the beginning of the buffer and get the length of the strings
            let buf_len = buf.len();
            
            let mut cursor: usize = 0;
            buf.set_rpos(cursor);
            let strings_len = u32::from_be(buf.read_u32());
            
            //NOTE: Here we are skipping the first 4 bytes, which are the lenght of the strings_buffer
            //      the strings_len which is the entirety of the strings_buffer
            let mut talent_cursor: usize = (4 + strings_len) as usize;
            
            //NOTE: Seek to the beggining of the entries (skip all strings)
            buf.set_rpos(talent_cursor);
            talent_cursor += 4;
            
            //TODO: use this for error checking the loop
            let talent_entries_count = u32::from_be(buf.read_u32());
            
            let mut loop_count = 0;
            //NOTE: Keep checking until you find it or run out of bytes
            loop
            {
                let name_idx = u32::from_be(buf.read_u32());
                
                if name_idx == 0 { panic!("name_idx is 0. Unrecoverable Parse error."); }
                
                buf.set_rpos(name_idx as usize);
                
                let name_len = u16::from_be(buf.read_u16());
                let name_vec = buf.read_bytes(name_len as usize);
                
                if name_len > 100 { println!("[{loop_count}] Invalid name_len: {name_len}"); panic!(); }
                
                let name_str = String::from_utf8(name_vec).expect("Invalid utf8 run?");
                let name_str = name_str.trim_end_matches("(Combattimento)").trim();
                let name_str = name_str.trim_end_matches("(Combattimento, Squadra)").trim();
                let name_str = name_str.trim_end_matches("(Combattimento, Critico)").trim();
                let name_str = name_str.trim_end_matches("(Combattimento, Occhiata)").trim();
                let name_str = name_str.trim_end_matches("(Creazione Oggetto)").trim();
                let name_str = name_str.trim_end_matches("(Creazione Oggetti)").trim();
                let name_str = name_str.trim_end_matches("(Metamagia)").trim();
                
                if name_str == needle {
                    let mut cached_index = CachedIndex { hash: hash, cursor: loop_count as u32 };
                    cache.push(cached_index);
                    
                    return loop_count as u32;
                }
                
                talent_cursor += TALENT_ENTRY_SIZE_IN_BYTES;
                if talent_cursor >= buf_len {
                    println!("Talent {needle} for {page_addr} wasn't found in the module");
                    return TALENT_NOT_FOUND;
                }
                
                buf.set_rpos(talent_cursor);
                
                loop_count += 1;
            }
        }
        
        (hash, found) => { return found; }
    }
    
    panic!("find_in_talents: Unreachable");
}

fn map_talent_intern_name(block: &str, name: &str, cache: &mut VectorCache, 
                          bufs: &mut Buffer_Context, talentsModule: &mut ByteBuffer, page_addr: &str) -> u32
{
    let mut effective_name = block;
    if name != "" { effective_name = name; }
    
    let mut is_mithic = effective_name.contains('\u{1d39}');
    let mut is_bonus  = effective_name.contains('\u{1d2e}');
    
    effective_name = effective_name.trim_end_matches(&['\u{1d39}']).trim(); //Superscript M
    effective_name = effective_name.trim_end_matches(&['\u{2e34}']).trim(); //Superscript ,
    effective_name = effective_name.trim_end_matches(&['\u{1d2e}']).trim(); //Superscript B
    
    let mut index_in_module: u32 = 0u32;
    if is_mithic {
        let mut mithic_name = effective_name.to_string();
        mithic_name.push_str(" (Mitico)");
        index_in_module = find_in_talents(&mut cache.talentsMod, talentsModule, &mithic_name, page_addr);
    }
    else { index_in_module = find_in_talents(&mut cache.talentsMod, talentsModule, effective_name, page_addr); }
    
    let mut name_intern_index = 0u16;
    let mut intern_bit        = 0u32;
    
    //NOTE: If name is missing, it means there's an extra '(XXX)' part. So we need to intern.
    if name != "" {
        //NOTE: We are doing this to completely remove annoying unicode from the compendium file.
        //      This data will be restored using the flags: is_mithic and is_bonus.
        let mut trimmed_block = block.replace(&['\u{1d39}', '\u{2e34}', '\u{1d2e}'], "");
        
        name_intern_index = add_entry_if_missing(&mut cache.talents, &mut bufs.talents, &trimmed_block);
        intern_bit        = TALENT_INTERN_BIT_U32;
    }
    
    let mut bonus_bit = 0u32;
    if is_bonus { bonus_bit = TALENT_BONUS_BIT_U32; }
    
    let mut mithic_bit = 0u32;
    if is_mithic { mithic_bit = TALENT_MITHIC_BIT_U32; }
    
    let mut result_idx: u32 = intern_bit;
    result_idx |= bonus_bit;
    result_idx |= mithic_bit;
    
    let shifted_name_intern_idx: u32 = (name_intern_index as u32) << TALENT_INTERN_IDX_SHIFT;
    result_idx |= shifted_name_intern_idx;
    
    result_idx |= index_in_module;
    
    /*
    let result_idx: u32 = intern_bit 
        | bonus_bit 
        | mithic_bit 
        | (name_intern_index as u32) << TALENT_INTERN_IDX_SHIFT
        | index_in_module;
    */
    return result_idx;
}

pub fn prepare_talents_str(stats: &mut Vec<&str>, cache: &mut VectorCache, bufs: &mut Buffer_Context,
                           talentsModule: &mut ByteBuffer, page_addr: &str) -> [u32; 24]
{
    if stats[TALENTS_IDX_IN_ARR] == "" { return [0u32; 24]; }
    if stats[TALENTS_IDX_IN_ARR] == "-" { return [0u32; 24]; }
    
    let mut result_arr = [0u32; 24];
    let mut result_idx = 0;
    
    //NOTE: Removing from the array doesn't work, because the elements are precisely placed. So we copy
    let trim_check: &[_] = &[' ', ',', ';', '.'];
    let base = stats[TALENTS_IDX_IN_ARR].trim_matches(trim_check);
    
    let mut talent_block = "";
    let mut talent_name  = "";
    
    let mut talent_start_idx = 0;
    let mut char_iter = base.char_indices();
    while let iter_opt = char_iter.next()
    {
        match iter_opt
        {
            Some((base_idx, char)) => {
                match char
                {
                    //NOTE: We found the end of this specific talent entry
                    ',' => {
                        talent_block = &base[talent_start_idx..base_idx].trim();
                        talent_start_idx = base_idx + 1;
                        
                        let talent_idx = map_talent_intern_name(&talent_block, &talent_name, cache, 
                                                                bufs, talentsModule, page_addr);
                        
                        result_arr[result_idx] = talent_idx;
                        result_idx += 1;
                        
                        talent_block = "";
                        talent_name  = "";
                    }
                    
                    
                    //NOTE: We found a specialization/condition for this talent
                    //      We need to skip until the end of the paren_block
                    '(' => {
                        talent_name = &base[talent_start_idx..base_idx].trim();
                        
                        let mut depth = 1;
                        while let Some((paren_idx, paren_char)) = char_iter.next()
                        {
                            match paren_char
                            {
                                '(' => { depth += 1; }
                                ')' => {
                                    depth -= 1;
                                    if depth == 0 { break; }
                                }
                                
                                _ => {}
                            }
                        }
                    }
                    
                    _ => { }
                }
            }
            
            //NOTE: Last element doesn't end with a comma, so we add it here.
            None =>
            {
                talent_block = &base[talent_start_idx..].trim();
                let talent_idx = map_talent_intern_name(&talent_block, &talent_name, cache, 
                                                        bufs, talentsModule, page_addr);
                result_arr[result_idx] = talent_idx;
                result_idx += 1;
                
                break;
            }
        }
    }
    
    return result_arr;
}