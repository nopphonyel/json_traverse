use core::fmt;
use std::collections::HashMap;

#[derive(Debug)]
pub enum JSON {
    Int(i64),
    Flt(f64),
    Str(String),
    Lst(Vec<JSON>),
    Obj(HashMap<String, JSON>),
    Bol(bool),
    Nul,
}

#[derive(Debug)]
enum Preb {
    BgnObj,
    BgnLst,
    Val(JSON),
    Key(String),
    Ent(String, JSON),
}

#[derive(PartialEq)]
enum Inside {
    Bgn,  // 0
    End,  // 1
    Obj,  // 2
    List, // 3
}

#[derive(PartialEq)]
enum S {
    Ready,     // 0
    ExpectKey, // 1
    BgnKey,    // 2
    EndKey,    // 3
    ExpectVal, // 4
    BgnPrimV,  // 5
    EndPrimV,  // 6
    BgnStrV,   // 7: This is for the esc char case
    EndStrV,   // 8
               //EndCtnr,   // 9
}

fn inside_what(mem: &Vec<Preb>) -> Inside {
    // Find what current inside state is
    let mem_size = mem.len();
    for i in (0..mem_size).rev() {
        match mem[i] {
            Preb::BgnLst => {
                return Inside::List;
            }
            Preb::BgnObj => {
                return Inside::Obj;
            }
            _ => {}
        }
    }
    Inside::End
}

fn get_esc_char(ch: char) -> Result<String, &'static str> {
    match ch {
        'b' => Ok(String::from(r"\b")),
        'f' => Ok(String::from(r"\f")),
        'n' => Ok(String::from("\n")),
        't' => Ok(String::from("\t")),
        'r' => Ok(String::from("\r")),
        '\\' => Ok(String::from("\\")),
        '\"' => Ok(String::from("\"")),
        _ => Err("Unavailable Escape Character"),
    }
}

// Memory updater function
fn pack_object(mem: &mut Vec<Preb>) -> Result<(), &'static str> {
    let mut temp_obj: HashMap<String, JSON> = HashMap::new();
    while let Some(preb) = mem.pop() {
        match preb {
            Preb::Ent(k, v) => {
                temp_obj.insert(String::from(k), v);
            }
            Preb::BgnObj => {
                break; // Go out from loop since there is no more to packup
            }
            _ => {
                return Err("There are some leftovers which not been processed yet...");
            }
        }
    }
    mem.push(Preb::Val(JSON::Obj(temp_obj)));
    Ok(())
}

fn pack_list(mem: &mut Vec<Preb>) -> Result<(), &'static str> {
    let mut temp_list: Vec<JSON> = Vec::new();
    while let Some(preb) = mem.pop() {
        match preb {
            Preb::BgnObj => {
                return Err("Unexpected '{' Begin of Object token was found!");
            }
            Preb::BgnLst => {
                mem.push(Preb::Val(JSON::Lst(temp_list)));
                return Ok(());
            }
            Preb::Val(v) => {
                temp_list.push(v);
            }
            Preb::Key(_) => {
                return Err("A key shouldn't be exist in side the list");
            }
            Preb::Ent(_, _) => {
                return Err("An entry shouldn't be exist in side the list");
            }
        }
    }
    Ok(())
}

fn pack_entry(mem: &mut Vec<Preb>) -> Result<(), &'static str> {
    // pop 2 element where the first will be value and the next will be key
    let mut val: JSON = JSON::Nul;
    if let Some(val_preb) = mem.pop() {
        match val_preb {
            Preb::Val(v) => {
                val = v;
            }
            _ => {
                return Err("Expected to be value.");
            }
        };
    } else {
        return Err("Stack mem is empty, Possibly invalid JSON format.");
    }

    let mut key: String = String::new();
    if let Some(key_preb) = mem.pop() {
        match key_preb {
            Preb::Key(k) => {
                key = String::from(k);
            }
            _ => {
                return Err("Expected to be key.");
            }
        };
    } else {
        return Err("Stack mem is empty, Possibly invalid JSON format.");
    }
    mem.push(Preb::Ent(key, val));
    Ok(())
}

fn primitive_parse(val_str: &String) -> Result<JSON, &'static str> {
    if val_str == "null" {
        Ok(JSON::Nul)
    } else if val_str == "true" {
        Ok(JSON::Bol(true))
    } else if val_str == "false" {
        Ok(JSON::Bol(false))
    } else {
        if val_str.len() >= 2 {
            if val_str.chars().nth(0).unwrap() == '0' && val_str.chars().nth(1).unwrap() == '0' {
                return Err("Too many 0 on front!");
            }
        }
        // Must be a number then
        if let Ok(int) = val_str.parse::<i64>() {
            return Ok(JSON::Int(int));
        }
        return if let Ok(flt) = val_str.parse::<f64>() {
            Ok(JSON::Flt(flt))
        } else {
            Err("Unparsable prmitive data... sorry")
        };
    }
}

// My custom error
#[derive(Debug)]
pub struct ParseErr {
    line_idx: usize,
    ch_pos: usize,
    msg: String,
}

impl ParseErr {
    fn e(lidx: usize, chpos: usize, err_msg: String) -> ParseErr {
        ParseErr {
            line_idx: lidx,
            ch_pos: chpos,
            msg: err_msg,
        }
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Line[{}], Char[{}]: {}",
            self.line_idx, self.ch_pos, self.msg
        )
    }
}

pub fn parse(json_str: &String) -> Result<JSON, ParseErr> {
    let mut mem: Vec<Preb> = Vec::new();
    let mut state: (Inside, S) = (Inside::Bgn, S::Ready);

    let mut temp_key: Option<String> = None;
    let mut temp_val: Option<String> = None;

    let mut esc_ch: bool = false; // Entering Escape Character mode

    let mut ch_pos: usize = 0;
    let mut line_idx: usize = 1;
    for ch in json_str.chars() {
        // Character and line counter
        match ch {
            '\n' => {
                line_idx += 1;
                ch_pos = 1;
            }
            _ => {
                ch_pos += 1;
            }
        }

        // Debug zone
        if line_idx == 11 && ch != ' ' {
            println!("Start DBG Mode!");
        }

        match state.0 {
            Inside::Bgn => {
                match ch {
                    '{' => {
                        mem.push(Preb::BgnObj);
                        state.0 = Inside::Obj;
                    }
                    '[' => {
                        mem.push(Preb::BgnLst);
                        state.0 = Inside::List;
                    }
                    ' ' | '\n' | '\r' | '\t' => { /* Do nothing ...*/ }
                    _ => {
                        return Err(ParseErr::e(
                            line_idx,
                            ch_pos,
                            "Expected \'{{\' or \'[\'.".into(),
                        ));
                    }
                }
            }
            Inside::Obj => {
                match state.1 {
                    S::Ready => {
                        match ch {
                            ' ' | '\n' | '\r' | '\t' => {} // Do nothing if space bar is entered
                            '}' => {
                                // End of object, try to pack up the previous entry
                                if let Err(err_msg) = pack_object(&mut mem) {
                                    return Err(ParseErr::e(line_idx, ch_pos, err_msg.into()));
                                }
                                state.0 = inside_what(&mem);
                                state.1 = S::EndPrimV;
                            }
                            '\"' => {
                                state.1 = S::BgnKey;
                                temp_key = Some(String::new());
                            }
                            _ => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Expected a String value as key.".into(),
                                ));
                            }
                        }
                    }
                    S::ExpectKey => match ch {
                        ' ' | '\n' | '\r' | '\t' => {}
                        '\"' => {
                            state.1 = S::BgnKey;
                            temp_key = Some(String::new());
                        }
                        _ => {
                            return Err(ParseErr::e(
                                line_idx,
                                ch_pos,
                                "Expected a String value as key.".into(),
                            ));
                        }
                    },
                    S::BgnKey => {
                        if esc_ch {
                            match get_esc_char(ch) {
                                Ok(esc_char) => {
                                    if let Some(tk) = &mut temp_key {
                                        tk.push_str(&esc_char);
                                    } else {
                                        return Err(ParseErr::e(
                                            line_idx,
                                            ch_pos,
                                            "String key not yet initialize.".into(),
                                        ));
                                    }
                                }
                                Err(err_msg) => {
                                    return Err(ParseErr::e(line_idx, ch_pos, err_msg.into()));
                                }
                            }
                            esc_ch = false;
                        } else {
                            match ch {
                                '\\' => {
                                    esc_ch = true;
                                }
                                '\"' => {
                                    state.1 = S::EndKey;
                                    if let Some(tk) = &mut temp_key {
                                        mem.push(Preb::Key(tk.clone()));
                                    } else {
                                        return Err(ParseErr::e(line_idx, ch_pos, "String key not yet initialize, but its was ending of key...?".into()));
                                    }
                                    temp_key = None;
                                }
                                _ => {
                                    if let Some(tk) = &mut temp_key {
                                        tk.push(ch);
                                    } else {
                                        return Err(ParseErr::e(line_idx, ch_pos, "String key not yet initialize, but its there was a character coming????".into()));
                                    }
                                }
                            }
                        }
                    }
                    S::EndKey => {
                        match ch {
                            ':' => {
                                state.1 = S::ExpectVal;
                            }
                            ' ' | '\t' | '\r' | '\n' => {}
                            _ => {
                                // Crash here because of unexpected character
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Expected ':' followed by value of given key".into(),
                                ));
                            }
                        }
                    }
                    S::ExpectVal => match ch {
                        '{' => {
                            state.0 = Inside::Obj;
                            state.1 = S::Ready;
                            mem.push(Preb::BgnObj);
                        }
                        '[' => {
                            state.0 = Inside::List;
                            state.1 = S::Ready;
                            mem.push(Preb::BgnLst);
                        }
                        '\"' => {
                            state.1 = S::BgnStrV;
                            temp_val = Some(String::new());
                        }
                        '0'..='9' | '-' | 't' | 'f' | 'n' => {
                            state.1 = S::BgnPrimV;
                            temp_val = Some(String::from(ch));
                        }
                        ' ' | '\t' => {}
                        _ => {
                            return Err(ParseErr::e(
                                line_idx,
                                ch_pos,
                                "Expected a Primitive Value!".into(),
                            ));
                        }
                    },
                    S::BgnPrimV => {
                        match ch {
                            ',' => {
                                // Now we're in Obj, we can pack the entry
                                if let Some(tv) = &temp_val {
                                    match primitive_parse(&tv) {
                                        Ok(pv) => {
                                            mem.push(Preb::Val(pv));

                                            // Pack entry imedietly
                                            if let Err(err_msg) = pack_entry(&mut mem) {
                                                return Err(ParseErr::e(
                                                    line_idx,
                                                    ch_pos,
                                                    err_msg.into(),
                                                ));
                                            }
                                        }
                                        Err(err_msg) => {
                                            return Err(ParseErr::e(
                                                line_idx,
                                                ch_pos,
                                                err_msg.into(),
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Value is not yet initialize".into(),
                                    ));
                                }
                                // If everything pass, safe to reset the temp_val
                                temp_val = None;
                                state.1 = S::ExpectKey;
                            }
                            '}' => {
                                // This imply that it is the end of object
                                // What we do is: parse value, packup entry, packup object,
                                //      change state of state.0 to result of inside_what(&mem)
                                // Example { ... "k12":true}
                                // However, there should be a verification that we're currently in Obj
                                // ^ No need since state.0 already know where we are

                                if let Some(tv) = &temp_val {
                                    match primitive_parse(&tv) {
                                        Ok(pv) => {
                                            mem.push(Preb::Val(pv));
                                            if let Err(err_msg) = pack_entry(&mut mem) {
                                                return Err(ParseErr::e(
                                                    line_idx,
                                                    ch_pos,
                                                    err_msg.into(),
                                                ));
                                            }
                                            if let Err(err_msg) = pack_object(&mut mem) {
                                                return Err(ParseErr::e(
                                                    line_idx,
                                                    ch_pos,
                                                    err_msg.into(),
                                                ));
                                            }
                                        }
                                        Err(err_msg) => {
                                            return Err(ParseErr::e(
                                                line_idx,
                                                ch_pos,
                                                err_msg.into(),
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Value is not yet initialize".into(),
                                    ));
                                }
                                temp_val = None;
                                // Update where we are
                                state.0 = inside_what(&mem);
                                state.1 = S::EndPrimV;
                            }
                            ']' => {
                                // This also imply that it is the end of list
                                // Example: .... -12, 34]
                                // Same as '}' but we check that we're in Lst
                                // Since we're in Inside:Obj -> just crash!
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Unexpected ']'! You are inside an Object, not a List!"
                                        .into(),
                                ));
                            }
                            ' ' | '\n' | '\r' | '\t' => {
                                // I think we safe to pack entry here
                                if let Some(tv) = &temp_val {
                                    match primitive_parse(&tv) {
                                        Ok(pv) => {
                                            mem.push(Preb::Val(pv));
                                            if let Err(err_msg) = pack_entry(&mut mem) {
                                                return Err(ParseErr::e(
                                                    line_idx,
                                                    ch_pos,
                                                    err_msg.into(),
                                                ));
                                            }
                                        }
                                        Err(err_msg) => {
                                            return Err(ParseErr::e(
                                                line_idx,
                                                ch_pos,
                                                err_msg.into(),
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Value is not given".into(),
                                    ));
                                }
                                temp_val = None;
                                state.1 = S::EndPrimV;
                            }
                            _ => {
                                if let Some(tv) = &mut temp_val {
                                    tv.push(ch);
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Primitive value is not yet initialized".into(),
                                    ));
                                }
                            }
                        }
                    }
                    S::EndPrimV => {
                        // This state will occur when after parsing the primitive value
                        // In other words, only when found the ' ', '\t' or '\n'
                        // Check that top of stack is entry already... if not then pack it
                        if let Some(preb) = &mem.last() {
                            match preb {
                                Preb::Ent(_, _) => {}
                                Preb::Val(_) => {
                                    if let Err(err_msg) = pack_entry(&mut mem) {
                                        return Err(ParseErr::e(
                                            line_idx,
                                            ch_pos,
                                            err_msg.into(),
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Expected Value".into(),
                                    ));
                                }
                            }
                        }

                        match ch {
                            ',' => {
                                // Maybe safe to jump to start
                                // if let Res::Fail(err_msg) = pack_entry(&mut mem) {
                                //     err_panic(line_idx, ch_pos, err_msg);
                                // }
                                state.1 = S::ExpectKey;
                            }
                            '}' => {
                                // The problem might occur when entry is already packed
                                // This imply that it is the end of object
                                // What we do is: just pack the object (Packing entry is done when found ' ', '\n', '\t' on  previous state)
                                // Example { ... "k12": true }

                                // Check the top of mem stack before pack object

                                if let Err(err_msg) = pack_object(&mut mem) {
                                    return Err(ParseErr::e(line_idx, ch_pos, err_msg.into()));
                                }
                                // Update where we are
                                state.0 = inside_what(&mem);
                                state.1 = S::EndPrimV; // Objects are count as Primitive Value
                            }
                            ']' => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Unexpected ']'! You are inside an Object, not a List!"
                                        .into(),
                                ));
                            }
                            ' ' | '\t' | '\r' | '\n' => {} // Ignore case
                            _ => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Expected '}' to finish the object".into(),
                                ));
                            }
                        }
                    }
                    S::BgnStrV => {
                        if esc_ch {
                            if let Some(tv) = &mut temp_val {
                                match get_esc_char(ch) {
                                    Ok(esc_char) => {
                                        tv.push_str(&esc_char);
                                    }
                                    Err(err_msg) => {
                                        return Err(ParseErr::e(
                                            line_idx,
                                            ch_pos,
                                            err_msg.into(),
                                        ));
                                    }
                                }
                            } else {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "String value is not yet initialized".into(),
                                ));
                            }
                            esc_ch = false;
                        } else {
                            match ch {
                                '\\' => {
                                    esc_ch = true;
                                }
                                '\"' => {
                                    // push string value to the mem
                                    // safe to pack entry
                                    if let Some(tv) = &temp_val {
                                        mem.push(Preb::Val(JSON::Str(tv.clone())));
                                    } else {
                                        return Err(ParseErr::e(line_idx, ch_pos,"Can't saved to mem since string value is not yet initialized".into()));
                                    }

                                    if let Err(err_msg) = pack_entry(&mut mem) {
                                        return Err(ParseErr::e(
                                            line_idx,
                                            ch_pos,
                                            err_msg.into(),
                                        ));
                                    } else {
                                        temp_val = None;
                                    }

                                    state.1 = S::EndStrV;
                                }
                                _ => {
                                    // keep pushing the ch to temp_value
                                    if let Some(tv) = &mut temp_val {
                                        tv.push(ch);
                                    } else {
                                        return Err(ParseErr::e(
                                            line_idx,
                                            ch_pos,
                                            "String value is not yet initialize".into(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    S::EndStrV => {
                        match ch {
                            '}' => {
                                // Example case: { ... ,"key1": "value" }
                                if let Err(err_msg) = pack_object(&mut mem) {
                                    return Err(ParseErr::e(line_idx, ch_pos, err_msg.into()));
                                }
                                state.0 = inside_what(&mem);
                                state.1 = S::EndPrimV;
                            }
                            ',' => {
                                // Example case: { ... ,"key1": "value", ... }
                                state.1 = S::ExpectKey;
                            }
                            ' ' | '\t' | '\r' | '\n' => {} // ignore case
                            _ => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Unexpected any character after end the String value"
                                        .into(),
                                ));
                            }
                        }
                    }
                }
            }
            Inside::List => {
                match state.1 {
                    S::Ready => match ch {
                        '{' => {
                            state.0 = Inside::Obj;
                            state.1 = S::Ready;
                            mem.push(Preb::BgnObj);
                        }
                        '[' => {
                            state.0 = Inside::List;
                            state.1 = S::Ready;
                            mem.push(Preb::BgnLst);
                        }
                        ']' => {
                            if let Err(err_msg) = pack_list(&mut mem) {
                                return Err(ParseErr::e(line_idx, ch_pos, err_msg.into()));
                            }
                            state.0 = inside_what(&mem);
                            state.1 = S::EndPrimV;
                        }
                        '\"' => {
                            state.1 = S::BgnStrV;
                            temp_val = Some(String::new());
                        }
                        '0'..='9' | '-' | 't' | 'f' | 'n' => {
                            state.1 = S::BgnPrimV;
                            temp_val = Some(String::from(ch));
                        }
                        ' ' | '\t' | '\r' | '\n' => {}
                        _ => {
                            return Err(ParseErr::e(
                                line_idx,
                                ch_pos,
                                "Expected value to be number, string, true, false or null"
                                    .into(),
                            ));
                        }
                    },
                    S::ExpectKey => {
                        return Err(ParseErr::e(
                            line_idx,
                            ch_pos,
                            "state:ExpectKey is not allowed!".into(),
                        ));
                    }
                    S::BgnKey => {
                        return Err(ParseErr::e(
                            line_idx,
                            ch_pos,
                            "state:BgnKey is not allowed!".into(),
                        ));
                    }
                    S::EndKey => {
                        return Err(ParseErr::e(
                            line_idx,
                            ch_pos,
                            "state:EndKey is not allowed!".into(),
                        ));
                    }
                    S::ExpectVal => match ch {
                        '{' => {
                            state.0 = Inside::Obj;
                            state.1 = S::Ready;
                            mem.push(Preb::BgnObj);
                        }
                        '[' => {
                            state.0 = Inside::List;
                            state.1 = S::Ready;
                            mem.push(Preb::BgnLst);
                        }
                        '\"' => {
                            state.1 = S::BgnStrV;
                            temp_val = Some(String::new());
                        }
                        '0'..='9' | '-' | 't' | 'f' | 'n' => {
                            state.1 = S::BgnPrimV;
                            temp_val = Some(String::from(ch));
                        }
                        ' ' | '\t' | '\r' | '\n' => {}
                        _ => {
                            return Err(ParseErr::e(
                                line_idx,
                                ch_pos,
                                "Expected value to be number, string, true, false or null"
                                    .into(),
                            ));
                        }
                    },
                    S::BgnPrimV => {
                        match ch {
                            ',' => {
                                // Now we're in List, we just parse and put in mem
                                if let Some(tv) = &temp_val {
                                    match primitive_parse(&tv) {
                                        Ok(pv) => {
                                            mem.push(Preb::Val(pv));
                                        }
                                        Err(err_msg) => {
                                            return Err(ParseErr::e(
                                                line_idx,
                                                ch_pos,
                                                err_msg.into(),
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Value is not given".into(),
                                    ));
                                }
                                temp_val = None;
                                state.1 = S::ExpectVal;
                            }
                            '}' => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Unexpected '}', currently inside a list not an Object!"
                                        .into(),
                                ));
                            }
                            ']' => {
                                // Example case [... 13, 12]
                                if let Some(tv) = &temp_val {
                                    match primitive_parse(tv) {
                                        Ok(pv) => {
                                            mem.push(Preb::Val(pv));
                                            if let Err(err_msg) = pack_list(&mut mem) {
                                                return Err(ParseErr::e(
                                                    line_idx,
                                                    ch_pos,
                                                    err_msg.into(),
                                                ));
                                            }
                                        }
                                        Err(err_msg) => {
                                            return Err(ParseErr::e(
                                                line_idx,
                                                ch_pos,
                                                err_msg.into(),
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Value is not yet initialized".into(),
                                    ));
                                }
                                temp_val = None;
                                // Update where we are
                                state.0 = inside_what(&mem);
                            }
                            ' ' | '\n' | '\r' | '\t' => { // TODO: Suspicious for bug!!
                                if let Some(tv) = &temp_val {
                                    match primitive_parse(&tv) {
                                        Ok(pv) => {
                                            mem.push(Preb::Val(pv));
                                            state.1 = S::EndPrimV;
                                            // if let Err(err_msg) = pack_entry(&mut mem) {
                                            //     return Err(ParseErr::e(
                                            //         line_idx,
                                            //         ch_pos,
                                            //         err_msg.to_string(),
                                            //     ));
                                            // }
                                        }
                                        Err(err_msg) => {
                                            return Err(ParseErr::e(
                                                line_idx,
                                                ch_pos,
                                                err_msg.into(),
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Value is not yet initialized".into(),
                                    ));
                                }
                                temp_val = None;
                                state.1 = S::EndPrimV;
                            }
                            _ => {
                                if let Some(tv) = &mut temp_val {
                                    tv.push(ch);
                                } else {
                                    return Err(ParseErr::e(
                                        line_idx,
                                        ch_pos,
                                        "Primitive value is not yet initialized".into(),
                                    ));
                                }
                            }
                        }
                    }
                    S::EndPrimV => {
                        match ch {
                            ',' => {
                                // Maybe safe to jump to start
                                state.1 = S::ExpectVal;
                            }
                            ']' => {
                                // This imply that it is the end of list
                                // What we do is: just pack the object
                                // Example [ ... "k12", true ]
                                if let Err(err_msg) = pack_list(&mut mem) {
                                    return Err(ParseErr::e(line_idx, ch_pos, err_msg.into()));
                                }
                                // Update where we are
                                state.0 = inside_what(&mem);
                                state.1 = S::EndPrimV; // Objects are count as Primitive Value
                            }
                            '}' => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Unexpected ']'! You are inside a List, not an Object!"
                                        .into(),
                                ));
                            }
                            ' ' | '\t' | '\r' | '\n' => {}
                            _ => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Expected ']' to finish the list or ',' for next value"
                                        .into(),
                                ));
                            }
                        }
                    }
                    S::BgnStrV => {
                        if esc_ch {
                            if let Some(tv) = &mut temp_val {
                                match get_esc_char(ch) {
                                    Ok(esc_char) => {
                                        tv.push_str(&esc_char);
                                    }
                                    Err(err_msg) => {
                                        return Err(ParseErr::e(
                                            line_idx,
                                            ch_pos,
                                            err_msg.into(),
                                        ));
                                    }
                                }
                            } else {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "String value is not yet initialized".into(),
                                ));
                            }
                            esc_ch = false;
                        } else {
                            match ch {
                                '\\' => {
                                    esc_ch = true;
                                }
                                '\"' => {
                                    // push string value to the mem
                                    // safe to pack entry
                                    if let Some(tv) = &temp_val {
                                        mem.push(Preb::Val(JSON::Str(tv.clone())));
                                    } else {
                                        return Err(ParseErr::e(line_idx, ch_pos,"Can't saved to mem since string value is not yet initialized".into()));
                                    }
                                    state.1 = S::EndStrV;
                                }
                                _ => {
                                    // keep pushing the ch to temp_value
                                    if let Some(tv) = &mut temp_val {
                                        tv.push(ch);
                                    } else {
                                        return Err(ParseErr::e(
                                            line_idx,
                                            ch_pos,
                                            "String value is not yet initialize".into(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    S::EndStrV => {
                        match ch {
                            ']' => {
                                // Example case: [ ... ,"value1", "value2" ]
                                if let Err(err_msg) = pack_list(&mut mem) {
                                    return Err(ParseErr::e(line_idx, ch_pos, err_msg.into()));
                                }
                                state.0 = inside_what(&mem);
                                state.1 = S::EndPrimV;
                            }
                            ',' => {
                                // Example case: [ ... ,"key1", "value", ... ]
                                state.1 = S::ExpectVal;
                            }
                            ' ' | '\t' | '\r' | '\n' => {} // ignore case
                            _ => {
                                return Err(ParseErr::e(
                                    line_idx,
                                    ch_pos,
                                    "Unexpect any char after end of the String value".into(),
                                ));
                            }
                        }
                    } 
                }
            }
            Inside::End => match ch {
                ' ' | '\t' | '\r' | '\n' => {}
                _ => {
                    return Err(ParseErr::e(
                        line_idx,
                        ch_pos,
                        "Any character after the end of root container is not allowed.".into(),
                    ));
                }
            },
        }
    }
    // Extract value from mem<Preb> to final_obj
    if state.0 == (Inside::End) {
        if mem.len() == 1 {
            if let Some(fobj) = mem.pop() {
                match fobj {
                    Preb::Val(final_object) => match final_object {
                        JSON::Lst(final_object) => Ok(JSON::Lst(final_object)),
                        JSON::Obj(final_object) => Ok(JSON::Obj(final_object)),
                        _ => Err(ParseErr::e(
                            line_idx,
                            ch_pos,
                            "Unexpected root JSON data type".into(),
                        )),
                    },
                    _ => Err(ParseErr::e(
                        line_idx,
                        ch_pos,
                        "Unexpected final tokens in parser memory".into(),
                    )),
                }
            } else {
                Err(ParseErr::e(
                    line_idx,
                    ch_pos,
                    "No data in parser memory".into(),
                ))
            }
        } else {
            Err(ParseErr::e(
                line_idx,
                ch_pos,
                "There is no or more than one JSON structure in a single file".into(),
            ))
        }
    } else {
        Err(ParseErr::e(
            line_idx,
            ch_pos,
            "Incomplete JSON structure".into(),
        ))
    }
}

fn pretty_print(obj: &JSON, lvl: usize, indent_str: &str) -> Result<(), String>{
    match obj {
        JSON::Lst(lst) => {
            if lvl == 0 {
                println!("[");
            }
            let len = lst.len();
            for i in 0..len {
                let e_item = lst.get(i).unwrap();
                match e_item {
                    JSON::Int(i) => { print!("{}{i}", indent_str.repeat(lvl+1)); },
                    JSON::Flt(f) => { print!("{}{f}", indent_str.repeat(lvl+1)); },
                    JSON::Str(s) => { print!("{}\"{s}\"", indent_str.repeat(lvl+1)); },
                    JSON::Lst(_) => {
                        println!("{}[", indent_str.repeat(lvl+1));
                        pretty_print(e_item, lvl+1, indent_str)?
                    },
                    JSON::Obj(_) => {
                        println!("{}{{", indent_str.repeat(lvl+1)); 
                        pretty_print(e_item, lvl+1, indent_str)?
                    },
                    JSON::Bol(b) => { print!("{}{:?}",indent_str.repeat(lvl+1), b); },
                    JSON::Nul => { print!("{}null", indent_str.repeat(lvl+1)); },
                }
                if i < len - 1 {
                    println!(",");
                } else {
                    println!("");
                }
            }
            print!("{}]", indent_str.repeat(lvl));
            Ok(())
        },
        JSON::Obj(obj) => {
            if lvl == 0 {
                println!("{{");
            }
            let mut elem_count = obj.len();
            for (k, v) in obj {
                elem_count -= 1;
                print!("{}\"{k}\": ", indent_str.repeat(lvl+1));
                match v {
                    JSON::Int(i) => { print!("{i}"); },
                    JSON::Flt(f) => { print!("{f}"); },
                    JSON::Str(s) => { print!("\"{s}\""); },
                    JSON::Lst(_) => {
                        println!("[");
                        pretty_print(v, lvl+1, indent_str)?
                    },
                    JSON::Obj(_) => { 
                        println!("{{");
                        pretty_print(v, lvl+1, indent_str)? 
                    },
                    JSON::Bol(b) => { print!("{:?}", b); },
                    JSON::Nul => { print!("null"); },
                }
                if elem_count > 0 {
                    println!(",");
                } else {
                    println!("");
                }
            }
            print!("{}}}", indent_str.repeat(lvl));
            Ok(())
        },
        _ => {
            Err("Expected a JSON list or object".into())
        },
    }
}

/*
[06/11/23]: We may need one more state which is called "EndCtnr" or "end container"
The containers are an Object or a List. This state has to do something different
which is when arrive this state, what it has to do is pack the entry first while other state
don't have any pre execution.

*/

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parse() {
        if let Ok(str_content) = fs::read_to_string("json_test/test.json") {
            println!("Testing json content : \n{str_content}");
            match parse(&str_content) {
                Ok(json) => {
                    match json {
                        JSON::Lst(obj) => {
                            println!("Found JSON List as root");
                            println!("{:?}", obj);
                        }
                        JSON::Obj(obj) => {
                            for each_key in obj.keys() {
                                println!("{each_key}");
                            }

                            if let Some(some_obj) = obj.get("results") {
                                if let JSON::Lst(vec) = some_obj {
                                    for each_element in vec {
                                        println!("{:?}", each_element);
                                    }
                                }
                            }
                            println!("Found JSON Object as root");
                            //println!("{:?}", obj);
                        }
                        _ => {}
                    }
                }
                Err(err_msg) => {
                    println!("{err_msg}");
                }
            }
        }
    }

    #[test]
    fn test_pretty_print(){
        if let Ok(str_content) = fs::read_to_string("json_test/test.json") {
            if let Ok(json_obj) = parse(&str_content){
                let r = pretty_print(&json_obj, 0, "  ");
                assert_eq!(r, Ok(()));
            }
        }
    }
}