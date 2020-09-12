use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;



fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    println!("In file {}", filename);
    let f = File::open(filename).expect("file not found");
    let reader = BufReader::new(f);

    for (_index, line) in reader.lines().enumerate() {
        let mut line = line.unwrap();
        if !line.contains("→"){
            continue;
        }
        let ll:Vec<&str> = line.split('→').collect();
        line = ll[ll.len()-1].to_string();
        let regex_rpn = regex_to_rpn(line);
        let mut hash_nfa: HashMap<u8, Vec<(char, u8)>> = HashMap::new();
        let (nfa_start, now_state) = build_nfa(regex_rpn, &mut hash_nfa);
        let hash_dfa = nfa_to_dfa(nfa_start, now_state, &mut hash_nfa);
        create_analyzer(hash_dfa);
    }
}

fn regex_to_rpn(mut regex:String) -> String{
    // todo: 文字が二つ続いた際に結合を意味する演算子?の挿入
    let mut rpn = "".to_string();
    let mut stack: Vec<char> = Vec::new();
    println!("{}", regex);
    regex = regex + "$";
    let mut priority:HashMap<char, u8> = HashMap::new();
    priority.insert('?', 5);
    priority.insert('|', 4);
    priority.insert('*', 6);
    priority.insert('(', 1);
    priority.insert(')', 1);
    for c in regex.chars(){
        match c {
            'l'|'n' => {
                rpn = rpn + &c.to_string();
            },
            '$' =>{
                while let Some(top) = stack.pop(){
                    rpn = rpn + &top.to_string();
                }
            },
            '(' => {
                stack.push(c)
            },
            ')' => {
                while let Some(top) = stack.pop(){
                    if top != '('{
                        rpn = rpn + &top.to_string();
                    }else{
                        break
                    }
                }
            }
            '?'|'*'|'|' => {
                while let Some(top) = stack.pop(){
                    println!("top:{},c:{}", top, c);
                    if priority[&c] <= priority[&top]{
                        rpn = rpn + &top.to_string();
                    }else{
                        stack.push(top);
                        break
                    }
                }
                stack.push(c);
            },
            _ => {
                //error
            },
        }
    }
    println!("{}", rpn);
    rpn

}

fn dfs_end_state(hash: &HashMap<u8, Vec<(char, u8)>>, state: u8) -> u8{
    let mut end_state:u8 = 0;
    if let Some(ch_to_vec) = hash.get(&state){
        for i in ch_to_vec {
            if (state - 1) != i.1{
                end_state = dfs_end_state(hash, i.1);
            }
        }
    }else{
        end_state = state;
    }
    end_state
}

fn dfs_connected_to_end(hash: &HashMap<u8, Vec<(char, u8)>>, state: u8, vec:&mut Vec<u8>){
    if let Some(ch_to_vec) = hash.get(&state){
        for i in ch_to_vec {
            if (state - 1) != i.1{
                if let Some(_) = hash.get(&i.1){
                    dfs_connected_to_end(hash, i.1, vec);
                }else{
                    vec.push(state);
                }
            }
        }
    }
}

fn build_symbol(hash:&mut HashMap<u8, Vec<(char, u8)>>, state: u8, c: char) -> u8{
    let mut transition_vec: Vec<(char, u8)> = Vec::new();
    let next_state = state + 1;
    println!("state:{},to_state:{}", state, next_state);
    transition_vec.push((c, next_state));
    hash.insert(state, transition_vec);
    next_state
}

fn build_union(nfa_left:u8, nfa_right:u8, state_number:u8, hash_nfa: &mut HashMap<u8, Vec<(char, u8)>>) -> u8{
    let mut new_start_vec:Vec<(char, u8)> = Vec::new();
    let mut new_end_vec_left:Vec<(char, u8)> = Vec::new();
    let mut new_end_vec_right:Vec<(char, u8)> = Vec::new();
    let new_start_state = state_number ;
    let new_end_state = state_number + 1;
    println!("union nfa1:{} and nfa2:{}", nfa_left, nfa_right);
    let end_state_left = dfs_end_state(hash_nfa, nfa_left);
    let end_state_right = dfs_end_state(hash_nfa, nfa_right);
    new_start_vec.push(('ε', nfa_left));
    new_start_vec.push(('ε', nfa_right));
    new_end_vec_left.push(('ε', new_end_state));
    new_end_vec_right.push(('ε', new_end_state));
    hash_nfa.insert(new_start_state, new_start_vec);
    hash_nfa.insert(end_state_left, new_end_vec_left);
    hash_nfa.insert(end_state_right, new_end_vec_right);
    new_end_state
}

fn build_clojure(nfa:u8, state_number:u8, hash_nfa: &mut HashMap<u8, Vec<(char, u8)>>) -> u8{
    // 終わりの状態、dfsで見つけるendの状態の命名修正
    println!("clojure nfa:{}", nfa);
    let new_start = state_number;
    let new_end = state_number + 1;
    let mut transition_vec1:Vec<(char, u8)> = Vec::new();
    transition_vec1.push(('ε', nfa));
    transition_vec1.push(('ε', new_end));
    hash_nfa.insert(new_start, transition_vec1);
    let end_state = dfs_end_state(hash_nfa, nfa);
    let mut transition_vec2: Vec<(char, u8)> = Vec::new();
    transition_vec2.push(('ε', nfa));
    transition_vec2.push(('ε', new_end));
    hash_nfa.insert(end_state, transition_vec2);
    new_end
}

fn build_concate(nfa_left:u8, nfa_right:u8, state_number:u8, hash_nfa: &mut HashMap<u8, Vec<(char, u8)>>) -> u8{
    // clojureを右側にして結合した場合、+1しないとnew_stateとendの差分が1で探索時に枝刈りされる
    let new_state_number = state_number + 1;
    println!("nfa_left:{}", nfa_left);
    let end_state = dfs_end_state(hash_nfa, nfa_left);
    println!("end_dfs:{}", end_state);
    let mut to_end_vec: Vec<u8> = Vec::new();
    dfs_connected_to_end(hash_nfa, nfa_left, &mut to_end_vec);
    for i in &to_end_vec{
        println!("element:{}", i);
    }
    to_end_vec.sort();
    to_end_vec.dedup();
    for i in &to_end_vec{
        let mut replace_end_vec: Vec<(char, u8)> = Vec::new();
        if let Some(vec) = hash_nfa.get_mut(&i){
            while let Some(top) = vec.pop(){
                if top.1 == end_state{
                    replace_end_vec.push((top.0, new_state_number));
                }else{
                    replace_end_vec.push((top.0, top.1));
                }
            }
        }
        println!("insert to state:{}", i);
        hash_nfa.insert(*i, replace_end_vec);
    }
    if let Some(vec) = hash_nfa.get_mut(&nfa_left){
        for _ in vec{
            println!("nfa_left end changed from:{} to:{}", end_state, new_state_number);
        }
    }

    let mut copied_vec: Vec<(char, u8)> = Vec::new();
    if let Some(vec) = hash_nfa.get_mut(&nfa_right){
        copied_vec = vec.to_vec()
    };
    hash_nfa.insert(new_state_number, copied_vec);
    hash_nfa.remove(&nfa_right);
    if let Some(vec) = hash_nfa.get_mut(&new_state_number){
        for _ in vec{
            println!("nfa_right start changed from:{}, to:{}", nfa_right, new_state_number);
        }
    }
    new_state_number
}


fn build_nfa(regex_rpn: String, mut hash_nfa: &mut HashMap<u8, Vec<(char, u8)>>) -> (u8, u8){
    let mut stack: Vec<u8> = Vec::new();
    let mut state_number = 1;
    for c in regex_rpn.chars(){
        if c == 'l' || c == 'n'{
            let next_state = build_symbol(&mut hash_nfa, state_number, c);
            stack.push(state_number);
            state_number = next_state + 1;
        }else{
            let nfa_left:u8;
            let nfa_right:u8;
            if let Some(top) = stack.pop(){
                nfa_right = top;
                if c == '*'{
                    println!("operator:{}, nfa_right:{}", c, nfa_right);
                    let end_state = build_clojure(nfa_right, state_number, &mut hash_nfa);
                    stack.push(state_number);
                    state_number = end_state + 1;
                }else{
                    if let Some(top) = stack.pop(){
                        nfa_left = top;
                        println!("operator:{}, nfa_left:{}, nfa_right:{}", c, nfa_left, nfa_right);
                        if c == '|'{
                            let end_state = build_union(nfa_left, nfa_right, state_number, &mut hash_nfa);
                            stack.push(state_number);
                            state_number = end_state + 1;
                        }else if c == '?'{
                            let end_state = build_concate(nfa_left, nfa_right, state_number, &mut hash_nfa);
                            stack.push(nfa_left);
                            state_number = end_state + 1;
                        }
                    }
                }
            }
        }
    }
    let mut ans_nfa = 0;
    while let Some(top) = stack.pop(){
        ans_nfa = top;
        println!("{}",top);
    }
    (ans_nfa, state_number)
}

fn full_print(hash: &mut HashMap<u8, Vec<(char, u8)>>){
    for i in 1..255{
        if let Some(vec) = hash.get(&i){
            for j in vec{
                println!("printing now:{}, c:{}, to:{}", i, j.0, j.1);
            }
        }
    }
}


fn dfs_after_epsilon(hash: &HashMap<u8, Vec<(char, u8)>>, state: u8, vec:&mut Vec<u8>){
    if let Some(ch_to_vec) = hash.get(&state){
        for i in ch_to_vec {
            if i.0 == 'ε'{
                vec.push(i.1);
                dfs_after_epsilon(hash, i.1, vec);
            }
        }
    }
}

fn nfa_to_dfa(start_state:u8, now_state: u8, hash: & HashMap<u8, Vec<(char, u8)>>) -> HashMap<u8, Vec<(char, u8)>>{
    let mut goto_table:HashMap<u8, HashMap<char, Vec<u8>>> = HashMap::new();
    // epsilonでない遷移先からepsilonだけで遷移できるものを求める
    for i in 1..now_state{
        if let Some(vec) = hash.get(&i){
            for j in vec{
                if j.0 != 'ε'{
                    let mut ep_after_vec:Vec<u8> = Vec::new();
                    dfs_after_epsilon(hash, j.1, &mut ep_after_vec);
                    ep_after_vec.push(j.1);
                    let mut ch_to_dockstate:HashMap<char, Vec<u8>> = HashMap::new();
                    ch_to_dockstate.insert(j.0, ep_after_vec);
                    goto_table.insert(i, ch_to_dockstate);
                }
            }
        }
    }
    for i in 1.. now_state{
        if let Some(h) = goto_table.get(&i){
            for (k,vec) in h{
                println!("state:{:?}, char:{}", i, k);
                for j in vec{
                    println!("docking state{:?}", j);
                }
            } 
        }
    }
    // 初期状態から行ける場所をまとめ初期状態とする
    let mut start_set = HashSet::new();
    let mut start_state_vec:Vec<u8> = Vec::new();
    dfs_after_epsilon(hash, start_state, &mut start_state_vec);
    start_set.insert(start_state);
    for i in &start_state_vec{
        start_set.insert(*i);
    }
    for j in &start_set{
        println!("start set:{:?}", j);
    }
    // 複数の状態をまとめて新しい集合を作る
    // let mut new_state = 100;
    let mut new_state = 1;
    let search_start = new_state;
    let mut all_chars: Vec<char> = Vec::new();
    all_chars.push('l');
    all_chars.push('n');
    let mut state_set_hash:HashMap<u8, HashSet<u8>> = HashMap::new();
    let mut new_state_hash:HashMap<u8, Vec<(char, u8)>> = HashMap::new();
    state_set_hash.insert(new_state, start_set);
    // あたらしい状態番号とそれに対応するhashは別に管理
    // 現在の集合の要素についてfor
    loop{
        let mut now_set = HashSet::new();
        if let Some(set) = state_set_hash.get(&(new_state)){
            for i in set{
                now_set.insert(*i);
            }
        }
        let mut now_state = new_state;
        println!("loop");
        let mut ch_to_vec:Vec<(char, u8)> = Vec::new();
        for c in &all_chars{
            let mut changed_flag = 0;
            let mut next_set = HashSet::new();
            // 全ての文字についてfor
            for i in &now_set{
                // goto_tableにその状態とその文字における遷移先の集合があるか確認
                if let Some(hashmap) = goto_table.get(&i){
                    if let Some(vec) = hashmap.get(&c){
                        // 集合があればそれを新しい集合にpush
                        changed_flag = 1;
                        for j in vec{
                            // println!("nowset:{}, c:{}, to:{}", i,c,j);
                            next_set.insert(*j);
                        }
                    }
                }
            }
            if changed_flag == 1{
                let mut set_flag = 1;
                let mut exists_state = 0;
                for i in search_start..now_state+1{
                    // println!("start:{:?}, now:{}",search_start, i);
                     if let Some(before_set) = state_set_hash.get(&i){
                        if before_set == &next_set{
                            println!("already exist: same as {}, now:{}",i, now_state);
                            exists_state = i;
                            set_flag = 0;
                        }
                    }
                }
                if set_flag == 1{
                    now_state = now_state + 1;
                    ch_to_vec.push((*c, now_state));
                    println!("next_set");
                    for i in &next_set{
                        println!("{:?}", i);
                    }
                    state_set_hash.insert(now_state, next_set);
                }else{
                    println!("add exists_state");
                    ch_to_vec.push((*c, exists_state));
                }
            }
        }
        if !ch_to_vec.is_empty(){
            let mut tmp_state = new_state + 1;
            while tmp_state != (now_state+1){
                println!("adding {}, {}", tmp_state, new_state);
                tmp_state += 1;
            }
            // println!("adding {}, {}", now_state, new_state);
            new_state_hash.insert(new_state, ch_to_vec);
            new_state += 1;
            full_print(&mut new_state_hash);
        }else{
            break;
        }
    }
    new_state_hash
}

fn create_analyzer(hash_dfa: HashMap<u8, Vec<(char, u8)>>){

    let temp_file = &"/Users/joya.kogo/private/analyzer_template.rs";

    let mut file = File::create(temp_file).unwrap();

    writeln!(&mut file, 
    "mod analyzer{{

    struct Analyzer {{
        state: u64
    }}

    impl Analyzer {{
        fn read_token(&mut self, ch:char) {{").unwrap();

    for i in 1..4{
        let st = "if self.state == ".to_owned()+&i.to_string()+"{";
        writeln!(&mut file, "{}", st).unwrap();
        writeln!(&mut file, "match ch {{");
        if let Some(vec) = hash_dfa.get(&i){
            for j in vec{
                let st2 = "'".to_string() + &j.0.to_string() + &"'".to_string() + &"=>" + &"self.state = " + &j.1.to_string() + &",";
                writeln!(&mut file, "{}", st2);
            }
            writeln!(&mut file, "_ => (),");
        }
        writeln!(&mut file, "}}");
        writeln!(&mut file, "}}");
    }

    writeln!(&mut file, "        }}
    }}

}}

fn main() {{

}}").unwrap();
}