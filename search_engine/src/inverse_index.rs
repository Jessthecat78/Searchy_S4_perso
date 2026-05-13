use std::collections::HashMap;

pub fn inverse_index(files:Vec<(usize,Vec<String>)>)->HashMap::<String, Vec<(i32,i32)>>{  
    let mut index = HashMap::<String, Vec<(i32,i32)>>::new();
    for (id,content) in files{
        let mut word_list = HashMap::<String,i32>::new();
        for word in content{
            if let Some(freq) = word_list.get_mut(&word){
                *freq+=1;

            }

            else {
                word_list.insert(word,1);
            }
    
        }
        for (key,value) in word_list{
            if let Some(w) = index.get_mut(&key){
                (*w).push((id as i32,value));
            }
            else{
                let mut nv = Vec::<(i32,i32)>::new();
                nv.push((id as i32,value));
                index.insert(key,nv);
            }


        }

    }
     return index;
        
}


