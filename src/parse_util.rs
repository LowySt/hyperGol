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

pub fn skip_to<'a>(data_slice: &'a str, tag: &str) -> &'a str
{
    let len = tag.len();
    let begin = data_slice.find(tag);
    
    if begin.is_none() { return GLOBAL_NULL; }
    
    let res = data_slice.get((begin.unwrap() + len)..);
    if res.is_none() { println!("[ERROR] Couldn't slice mob_page past tag {}", tag); return GLOBAL_NULL; }
    
    return res.unwrap();
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

pub fn clear_all_tags(data_slice: &str) -> String {
    
    let mut result = String::from(data_slice);
    loop
    {
        let begin = result.find("<");
        if begin.is_none() { return result.replace("&#160;", " "); }
        
        let end = result.find(">");
        if end.is_none() { println!("[ERROR] Malformed html?"); return "".to_string(); }
        
        if begin.unwrap() > end.unwrap() { println!("[ERROR] Bad assumption?"); return "".to_string(); }
        
        result.replace_range(begin.unwrap()..end.unwrap()+1, "");
    }
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
