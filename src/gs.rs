//NOTE: Here's the bit mapping of the u16 GS
//
//      [15..12]  [11..6]  [5..0]
//      ^           ^         ^
//      |           |       GS Value (from 0 to 35 including the fractionals)
//      |         RM Value from 0 to 10
//      |
//   Extra Bits
//
//   Bit 15 indicates whether or not the RM value is present!

const GS_RM_VALUE_BIT:      u16   = 0x8000;
const GS_FRACTIONAL_OFFSET: u16   = 4;

const GS_RM_BIT_OFFSET:     usize = 6;
const GS_SENTINEL_VALUE:    u16   = 0xFFFF;

fn map_simple_gs(field: &str, page_name: &str) -> u16
{
    if field.contains("/") {
        //NOTE: Fractional GS Creature
        match field {
            "1/8" => { return 0; }
            "1/6" => { return 1; }
            "1/4" => { return 2; }
            "1/3" => { return 3; }
            "1/2" => { return 4; }
            
            _ => { panic!("Invalid GS Fractional value from {field} in {page_name}"); }
        }
    }
    else {
        //NOTE: Integer GS Creature
        let gs: u16 = field.parse::<u16>()
            .unwrap_or_else(|_| panic!("Couldn't parse simple gs from {field} in {page_name}"));
        return (gs + GS_FRACTIONAL_OFFSET);
    }
}

pub fn map_gs(field: &str, page_name: &str) -> u16
{
    if field.len() == 0 { return GS_SENTINEL_VALUE; }
    
    if field.contains("/RM") {
        //NOTE: Mithical Rank Creature
        let collected: Vec<&str> = field.split("/RM").collect();
        
        let gs_val: u16 = map_simple_gs(collected[0], page_name);
        let rm_val: u16 = collected[1].trim().parse::<u16>()
            .unwrap_or_else(|_| panic!("Couldn't parse rm from {field} in {page_name}"));
        
        let result: u16 = GS_RM_VALUE_BIT | (rm_val << GS_RM_BIT_OFFSET) | gs_val;
        return result;
    }
    else {
        return map_simple_gs(field, page_name);
    }
}