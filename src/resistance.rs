use bytebuffer::ByteBuffer;

use crate::entries::*;
use crate::vec_cache::*;

//NOTE: Resitances are represented by a 4bit Type and a 6bit Value.
//      A single 64bit word contains 6 different resistances and 4 extra bits.
//
// [63..60]  [59..50]  [49..40]  [39..30]  [29..20]  [19..10]  [9..0]
//    ^         ^         ^         ^         ^         ^       ^
//    |         |         |         |         |         |     Resistance (4bit Type, 6bit Value)
//    |         |         |         |         |         |
//    |         |         |         |         |       Resistance (4bit Type, 6bit Value)
//    |         |         |         |         |
//    |         |         |         |       Resistance (4bit Type, 6bit Value)
//    |         |         |         |
//    |         |         |       Resistance (4bit Type, 6bit Value)
//    |         |         |         
//    |         |       Resistance (4bit Type, 6bit Value)
//    |         |
//    |       Resistance (4bit Type, 6bit Value)
//    |
//  Extra Bits
//
// The Extra bits are currently UNUSED!

const RES_SENTINEL_VALUE: u64   = 0xFFFFFFFFFFFFFFFF;
const RES_SINGLE_BIT_LEN: usize = 10;
const RES_MAX_COUNT:      usize = 6;

const RES_INVALID_TYPE:   u64 = 0x0000;
const RES_ACID_TYPE:      u64 = 0x0040;
const RES_FIRE_TYPE:      u64 = 0x0080;
const RES_COLD_TYPE:      u64 = 0x00C0;
const RES_ELEC_TYPE:      u64 = 0x0100;
const RES_SOUND_TYPE:     u64 = 0x0140;
const RES_POS_TYPE:       u64 = 0x0180;
const RES_NEG_TYPE:       u64 = 0x01C0;
const RES_CHAN_TYPE:      u64 = 0x0200;
const RES_ADAP_TYPE:      u64 = 0x0240;
const RES_ALL_TYPE:       u64 = 0x0280;
const RES_POS_50_TYPE:    u64 = 0x02C0;
const RES_PSN_TYPE:       u64 = 0x0300;
const RES_SNAKESKIN_TYPE: u64 = 0x0340;

const RES_MAX_VALUE: u64    = 0x3F;

fn parse_single_resistance(block: &str, page_name: &str) -> u64
{
    let mut result: u64 = 0u64;
    
    //NOTE: We expect a type and a value, so we split by space and match both
    let mut split = block.split(' ');
    
    let first = split.next();
    match(first)
    {
        Some("Acido") | Some("acido") => { result |= RES_ACID_TYPE; }
        Some("Fuoco") | Some("fuoco") => { result |= RES_FIRE_TYPE; }
        Some("Freddo") | Some("freddo") => { result |= RES_COLD_TYPE; }
        Some("Elettricità") | Some("elettricità") => { result |= RES_ELEC_TYPE; }
        
        Some("Sonoro") | Some("sonoro") => { result |= RES_SOUND_TYPE; }
        
        Some("Energia") | Some("Energie") | Some("energie") | Some("energia") => {
            let second = split.next();
            match(second)
            {
                Some("Positiva") | Some("positiva") => {
                    //NOTE: Check edge case for special type of resistance.
                    if block == "Energia Positiva (50%)" {
                        result |= RES_POS_50_TYPE;
                        return result;
                    }
                    
                    result |= RES_POS_TYPE;
                }
                Some("Negativa") | Some("negativa") => { result |= RES_NEG_TYPE; }
                Some("(Tutte)")                     => { result |= RES_ALL_TYPE; }
                
                _ => { todo!("Unhandled energy resistance Type: {block} in {page_name}"); }
            }
        }
        
        Some("Resistenza") | Some("resistenza") => {
            let second = split.next();
            
            match(second)
            {
                Some("Adattiva") | Some("adattiva") => { result |= RES_ADAP_TYPE; }
                
                Some("ad") => {
                    let third = split.next();
                    assert!(third == Some("Incanalare"), "Unhandled Resistance Type: {block} in {page_name}");
                    result |= RES_CHAN_TYPE;
                }
                
                //NOTETODO: This is not a resistance. I hate Villain Codex.
                //    Should I take a stand and move this to Defensive Cap, where it belongs?
                Some("al") => {
                    let third = split.next();
                    assert!(third == Some("Veleno"), "Unhandled Resistance Type: {block} in {page_name}");
                    result |= RES_PSN_TYPE;
                    return result;
                }
                
                _ => { todo!("Unhandled resistance Type: {block} in {page_name}"); }
            }
        }
        
        //NOTETODO: This is not a resistance. I hate Villain Codex.
        //    Should I take a stand and move this to Defensive Cap, where it belongs?
        Some("Pelle") => {
            assert!(block == "Pelle di Serpente", "Unhandled resistance Type: {block} in {page_name}");
            result |= RES_SNAKESKIN_TYPE;
            return result;
        }
        
        _ => { todo!("Unhandled resistance Type: {block} in {page_name}"); }
    }
    
    //NOTE: Next we find the value.
    let value = split.next().expect("Missing Resistance Value Split in {block} from {page_addr}").parse::<u64>()
        .unwrap_or_else(|_| panic!("Couldn't Parse Resistance Value in: {block} from {page_name}"));
    assert!(value <= RES_MAX_VALUE, "Value is larger than max value in {block} from {page_name}.");
    
    result |= value;
    
    return result;
}

pub fn prepare_and_map_resistances(defenses: &mut Vec<&str>, res_off: usize, page_name: &str) -> u64
{
    if defenses[RES_IDX_IN_ARR + res_off] == ""  { return RES_SENTINEL_VALUE; }
    if defenses[RES_IDX_IN_ARR + res_off] == "-" { return RES_SENTINEL_VALUE; }
    
    //NOTE: We remove all prefixes and suffixes that match the below check (because of golarion syntax errors)
    let trim_check: &[_] = &[' ', ',', ';', '.'];
    let base = defenses[RES_IDX_IN_ARR + res_off].trim_matches(trim_check);
    
    let mut result: u64 = 0u64;
    let mut res_count = 0;
    
    let mut res_start_idx = 0;
    let mut char_iter = base.char_indices().peekable();
    while let Some((base_idx, char)) = char_iter.next()
    {
        match char
        {
            //NOTE: We found the end of a resistance
            ',' => {
                assert!(res_count < RES_MAX_COUNT, "Too many resistances in {page_name}");
                
                //NOTE: Now we need to further parse the Type from the Value
                
                //NOTE: We know this is safe because res_star_idx and base_idx are validly aligned indices
                //      and `base_idx + 1` has to be valid because we just matched an ASCII ',' character.
                let single_res_block = &base[res_start_idx..base_idx].trim();
                res_start_idx = base_idx + 1;
                
                let single_res: u64 = parse_single_resistance(single_res_block, page_name);
                
                result |= (single_res << res_count*RES_SINGLE_BIT_LEN);
                res_count += 1;
            }
            
            //NOTE: Identifiers
            _ => {}
        }
    }
    
    //NOTE: Outside the while we should've hit the end of the string, so the last resistance has to be added
    let remainder = &base[res_start_idx..].trim();
    if remainder.len() > 2
    {
        assert!(res_count < RES_MAX_COUNT, "Too many resistances in {page_name}");
        
        let single_res: u64  = parse_single_resistance(remainder, page_name);
        result |= (single_res << res_count*RES_SINGLE_BIT_LEN);
        res_count += 1;
    }
    
    return result;
}