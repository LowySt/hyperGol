//TODO: Get rid of this.
pub static GLOBAL_NULL: &str = "";

pub fn flatten_str_list(orig_arr: &mut Vec<&str>, list_idx: usize, delim: &str) -> usize
{
    let mut number_of_inserts = 0;
    let mut base = orig_arr.remove(list_idx);
    let mut idx = list_idx;
    loop
    {
        let (new_el, next) = get_until(&base, delim);
        base = next;
        
        if new_el == GLOBAL_NULL { break; }
        orig_arr.insert(idx, new_el);
        number_of_inserts += 1;
        idx += 1;
    }
    
    let (last_el, _next) = get_until(&base, "");
    orig_arr.insert(idx, last_el);
    number_of_inserts += 1;
    
    return number_of_inserts;
}

pub fn get_until<'a>(data_slice: &'a str, until: &str) -> (&'a str, &'a str)
{
    let end_idx: usize;
    let tag_len = until.len();
    
    if until.len() == 0 { end_idx = data_slice.len(); }
    else
    {
        let index = data_slice.find(until);
        if index.is_none() { return (GLOBAL_NULL, data_slice) }
        
        end_idx = index.unwrap();
    }
    
    let result = data_slice.get(..end_idx);
    if result.is_none() { println!("[ERROR] Malformed mob page\n"); panic!(); }
    
    let tag_len = until.len();
    let next = data_slice.get(end_idx+tag_len..);
    if next.is_none() { println!("[ERROR] Malformed mob page\n"); panic!(); }
    
    return (result.unwrap(), next.unwrap());
}

//TODO: This should be replaced by skip_tagged_block
pub fn skip_to<'a>(data_slice: &'a str, tag: &str) -> &'a str
{
    let len = tag.len();
    let begin = data_slice.find(tag);
    
    if begin.is_none() { return GLOBAL_NULL; }
    
    let res = data_slice.get((begin.unwrap() + len)..);
    if res.is_none() { println!("[ERROR] Couldn't slice mob_page past tag {}", tag); return GLOBAL_NULL; }
    
    return res.unwrap();
}

pub fn skip_end_tag<'a>(data: &'a str) -> Option<&'a /*'*/str>
{
    let mut char_iter = data.char_indices();
    while let Some((char_idx, char)) = char_iter.next()
    {
        match char
        {
            '>' => {
                if data[char_idx..].len() > 2
                {
                    return Some(&data[char_idx+1..]);
                }
                else
                {
                    return None;
                }
            }
            
            _ => {}
        }
    }
    
    return None;
}

pub fn 
slice_between_inclusive<'a>(source: &'a str, start: &'a str, end: &'a str) -> Option<(&'a str, usize, usize/*'*/)>
{
    if let Some(start_idx) = source.find(start)
    {
        if let Some(end_idx) = source.find(end)
        {
            if(start_idx < end_idx)
            {
                return Some((&source[start_idx..end_idx+end.len()], start_idx, end_idx));
            }
            
            return None;
        }
        
        return None;
    }
    
    return None;
}

//NOTE: Works like split_at, but doesn't split an index, rather a delimiter
//      Like split_once, but is inclusive of the delimiter on the second slice
//TODO: Rename to something better, like split_once_inclusive, or split_at_delim
pub fn split_off<'a>(source: &'a str, delim: &'a str) -> Option<(&'a str, &'a str)>
{
    for c in delim.chars()
    {
        if let Some(idx) = source.find(c)
        {
            return Some((&source[..idx], &source[idx..]));
        }
    }
    
    return None;
}

//TODO: Rename to something better, like split_once_inclusive, or split_at_delim
pub fn split_off_idx<'a>(source: &'a str, delim: &'a str) -> Option<(&'a str, &'a str, usize)>
{
    for c in delim.chars()
    {
        if let Some(idx) = source.find(c)
        {
            return Some((&source[..idx], &source[idx..], idx));
        }
    }
    
    return None;
}


//NOTE: Left is value (from sign to last digit), Right is rest
//      This only works if the value is on the left side/right side of the base
pub fn split_at_value<'a>(base: &'a str) -> Option<(i16, &'a str/*'*/)>
{
    let mut value: &str = "";
    
    //TODO: Do I like checking for the unicode character (which will be stripped anyway), or should I strip it
    //      before hand? Probably better to strip later because I might not intern something here,
    //      So the early stripping would be extra, unnecessary work.
    if let Some((left, right, idx)) = split_off_idx(base, "+-\u{2013}")
    {
        let mut right_iter = right.char_indices().peekable();
        
        let (_, sign_c) = right_iter.next().unwrap(); //NOTE: Can't fail unwrap because split_off already checked it.
        let start_idx = sign_c.len_utf8();
        let mut end_idx = start_idx;
        let mut sign = 1;
        if sign_c == '-' || sign_c == '\u{2013}' { sign = -1; }
        while let Some((value_idx, value_char)) = right_iter.peek() //NOTE: char_indices not needed?
        {
            if !value_char.is_digit(10) { break; }
            
            //NOTE: Consume only if it's still part of the value.
            end_idx += value_char.len_utf8();
            right_iter.next();
        }
        
        if end_idx == start_idx { return None; } //NOTE: Means there's only the +/- character without anything else
        
        //NOTE: We parse the value right here, because of stupid unicode dashes
        //      Skip the sign, we know if it's negative with the boolean is_minus
        value = &right[start_idx..end_idx];
        
        if left.len() < 2
        {
            //NOTE: The value is on the left side of the base
            let rest = &right[end_idx..];
            let mut num_value: i16 = value.parse::<i16>()
                .unwrap_or_else(|_| panic!("[ERROR] couldn't parse value: {:#?}", value));
            num_value *= sign;
            
            assert!(num_value > -127 && num_value < 127, "Value out of range: {num_value}");
            
            return Some((num_value, rest));
        }
        else
        {
            if right.len() - end_idx > 2
            {
                //NOTE: Our value is right in the middle
                //      Maybe we just return None in this case
                return None;
            }
            
            let mut num_value: i16 = value.parse::<i16>()
                .unwrap_or_else(|_| panic!("[ERROR] couldn't parse value: {:#?}", value));
            num_value *= sign;
            
            assert!(num_value > -127 && num_value < 127, "Value out of range, {num_value}");
            
            //NOTE: The value is on the right side of the base
            return Some((num_value, left));
        }
    }
    
    //NOTE: No +/- sign was found.
    //TODO: There could technically be a value without sign. Maybe handle this, by splitting of at numeric?
    return None;
}

pub fn trim_str<'a>(base: &'a str, check: &[&str]) -> &'a str
{
    let mut result = base;
    for i in check
    {
        if let Some(a) = base.strip_prefix(i)
        {
            result = a;
        }
    }
    
    return result;
}

pub fn clear_all_tags(data_slice: &str) -> String {
    
    let mut result = String::with_capacity(data_slice.len());
    let mut char_iter = data_slice.char_indices();
    while let iter_opt = char_iter.next()
    {
        match iter_opt
        {
            Some((curr_idx, char)) => {
                match char
                {
                    '<' => {
                        let tag_test = &data_slice[curr_idx..];
                        
                        //NOTE: We substitute the sup tag text with special utf8 characters
                        //      So that PCMan knows they are supposed to be sup, and the rest of the system
                        //      doesn't mistake it for normal text.
                        if tag_test.starts_with("<sup>")
                        {
                            char_iter.nth(3);
                            
                            while let Some((sup_idx, sup_char)) = char_iter.next()
                            {
                                match sup_char {
                                    
                                    //NOTE: Bonus and Mithic, usually Talents
                                    'B' => { result.push_str("\u{1d2e}"); }
                                    ',' => { result.push_str("\u{2e34}"); }
                                    ' ' => { result.push_str("\u{2009}"); }
                                    'M' => { result.push_str("\u{1d39}"); }
                                    
                                    //NOTE: Dominio, usually for clerics/paladin's spells.
                                    'D' => { result.push_str("\u{1d30}"); }
                                    
                                    //NOTE: Stirpe, usually for sorcerers
                                    'S' => { result.push_str("\u{02e2}"); }
                                    
                                    //NOTE: Misc...
                                    'O' => { result.push_str("\u{1d3c}"); }
                                    'T' => { result.push_str("\u{1d40}"); }
                                    'U' => { result.push_str("\u{1d41}"); }
                                    'I' => { result.push_str("\u{1d35}"); }
                                    
                                    //NOTE: Usually appears in measurements (i.e. 30 cm^3, 2 dm^2)
                                    '1' => { result.push_str("\u{00b1}"); }
                                    '2' => { result.push_str("\u{00b2}"); }
                                    '3' => { result.push_str("\u{00b3}"); }
                                    
                                    //NOTE: No substitution. Why is it even a sup on golarion!?!?
                                    '*' => { result.push('*'); }
                                    
                                    //NOTE: The sup should end
                                    '<' => {
                                        let end_tag_test = &data_slice[sup_idx..];
                                        
                                        if !end_tag_test.starts_with("</sup>") {
                                            
                                            while let Some((inside_idx, non_sup)) = char_iter.next()
                                            { if non_sup == '>' { break; } }
                                        }
                                        else { char_iter.nth(4); break; }
                                    }
                                    
                                    _ => { todo!("Unhandled superscript character {sup_char} in clear_all_tags"); }
                                }
                            }
                        }
                        else //NOTE: We skip the entire tag
                        {
                            while let Some((tag_idx, tag_char)) = char_iter.next()
                            {
                                if tag_char == '>' { break; }
                            }
                        }
                    }
                    
                    _ => {
                        result.push(char);
                    }
                }
            }
            
            None => {
                return result.replace("&#160;", "");
            }
        }
    }
    
    panic!("unreachable? in clear_all_tags()");
    
    return result.replace("&#160;", "");
}

//TODO: Because I'm removing something that is possibly in the middle of a string,
//      I can't avoid constructing a new string to remove that range, but
//      Maybe I can find some smarter way to deal with this?
pub fn clear_tag(data_slice: &str, tag_begin: &str, tag_end: &str) -> String {
    
    let begin_idx = data_slice.find(tag_begin);
    let end_idx   = data_slice.find(tag_end);
    
    if begin_idx.is_none() { return "".to_string(); }
    if end_idx.is_none()   { return "".to_string(); }
    
    let mut result_slice = String::from(data_slice);
    result_slice.replace_range(begin_idx.unwrap()..end_idx.unwrap(), "");
    
    return result_slice;
}

//TODO: Remove this and use `get_tagged_block()`
pub fn get_slice_inside_tags<'a>(data_slice: &'a str, tag_begin: &str, tag_end: &str) -> (&'a str, &'a str) {
    
    let begin_idx = data_slice.find(tag_begin);
    let end_idx   = data_slice.find(tag_end);
    
    if begin_idx.is_none() { return ("", ""); }
    if end_idx.is_none()   { return ("", ""); }
    
    let result    = data_slice.get(begin_idx.unwrap()+tag_begin.len()..end_idx.unwrap());
    if result.is_none()    { return ("", ""); }
    
    let next_data = data_slice.get(end_idx.unwrap()+tag_end.len()..);
    if next_data.is_none()    { return ("", ""); }
    
    return (result.unwrap(), next_data.unwrap());
}

//NOTE: The left result is the extracted block, the right result is the rest
pub fn get_tagged_block<'a>(data: &'a str, tag_begin: &str, tag_end: &str, first_match: &str) -> Option<(&'a str, &'a str)>
{
    if let Some(idx) = data.find(first_match)
    {
        let beg_len   = tag_begin.len();
        let match_len = tag_end.len();
        let start_idx = idx + first_match.len();
        let mut depth = 0;
        
        let content = skip_end_tag(&data[start_idx..])?;
        let mut char_iter = content.char_indices().peekable();
        while let Some((char_idx, char)) = char_iter.peek()
        {
            match char
            {
                '<' => {
                    if content[*char_idx..].len() >= beg_len 
                    {
                        if let Some(sl) = content.get(*char_idx..*char_idx + beg_len)
                        {
                            if sl == tag_begin { depth += 1; char_iter.next(); continue; }
                        }
                    }
                    
                    if content[*char_idx..].len() >= match_len
                    {
                        if let Some(sl) = content.get(*char_idx..*char_idx + match_len)
                        {
                            if sl == tag_end
                            {
                                if depth == 0
                                {
                                    return Some((&content[..*char_idx], &content[*char_idx+match_len..]));
                                }
                                else
                                {
                                    depth -= 1;
                                }
                            }
                        }
                    }
                }
                
                _ => {}
            }
            
            char_iter.next();
        }
    }
    else
    {
        return None;
    }
    
    return None;
}

pub fn skip_tagged_block<'a>(data: &'a str, tag_begin: &str, tag_end: &str) -> Option<&'a /*'*/ str>
{
    if let Some(idx) = data.find(tag_begin)
    {
        let beg_len = tag_begin.len();
        let match_len = tag_end.len();
        let start_idx = idx + beg_len;
        let mut end_idx = start_idx;
        let mut depth = 0;
        
        let content = &data[start_idx..];
        let mut char_iter = content.char_indices().peekable();
        while let Some((char_idx, char)) = char_iter.peek()
        {
            match char
            {
                '<' => {
                    if content[*char_idx..].len() >= beg_len
                    {
                        if let Some(sl) = content.get(*char_idx..*char_idx + beg_len)
                        {
                            if sl == tag_begin { depth += 1; char_iter.next(); continue; }
                        }
                    }
                    
                    if content[*char_idx..].len() >= match_len 
                    {
                        if let Some(sl) = content.get(*char_idx..*char_idx + match_len)
                        {
                            if sl == tag_end
                            {
                                if depth == 0
                                {
                                    return Some(&content[*char_idx + match_len..]);
                                }
                                else
                                {
                                    depth -= 1;
                                }
                            }
                        }
                    }
                }
                
                _ => {}
            }
            
            char_iter.next();
        }
    }
    else
    {
        //NOTE: If the tag can't be found, it means there's no tagged block to be skipped.
        //      Can this function even return None? Should I just remove the option?
        return Some(data);
    }
    
    return None;
}

pub fn fill_array_from_available<'a>(data_slice: &'a str, until: &[&str]) -> Vec<&'a str>
{
    let mut result_arr = vec![];
    
    let mut next: &str = data_slice;
    let mut el: &str;
    let mut i: usize = 0;
    
    let mut missed_i: usize = 99;
    
    for i in 0..until.len()
    {
        (el, next) = get_until(next, until[i]);
        
        if el == GLOBAL_NULL && missed_i == 99 && i != 0 { missed_i = i; }
        
        if missed_i == 99 || el == GLOBAL_NULL { result_arr.push(el.trim()); }
        else                                   { result_arr.insert(missed_i, el.trim()); missed_i = 99; }
    }
    
    let (last, _) = get_until(next, "");
    if missed_i == 99 { result_arr.push(last.trim()); }
    else
    { 
        result_arr.insert(missed_i, last.trim());
        result_arr.push(GLOBAL_NULL); //Push the last missed element.
    }
    
    return result_arr;
}

pub fn fill_array_from_available_dbg<'a>(data_slice: &'a str, until: &[&str]) -> Vec<&'a str>
{
    let mut result_arr = vec![];
    
    let mut next: &str = data_slice;
    let mut el: &str;
    let mut i: usize = 0;
    
    let mut missed_i: usize = 99;
    
    for i in 0..until.len()
    {
        (el, next) = get_until(next, until[i]);
        
        if el == GLOBAL_NULL && missed_i == 99 && i != 0 { missed_i = i; }
        
        if missed_i == 99 || el == GLOBAL_NULL { result_arr.push(el.trim()); }
        else                                   { result_arr.insert(missed_i, el.trim()); missed_i = 99; }
        
        println!("{:#?}", result_arr);
    }
    
    let (last, _) = get_until(next, "");
    
    if missed_i == 99 { result_arr.push(last.trim()); }
    else
    { 
        result_arr.insert(missed_i, last.trim());
        result_arr.push(GLOBAL_NULL); //Push the last missed element.
    }
    
    println!("{:#?}", result_arr);
    
    return result_arr;
}
