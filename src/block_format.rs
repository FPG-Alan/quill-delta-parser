use std::{collections::HashMap};

#[derive(Clone)]
pub struct BlockTag {
    block_type: &'static str,
    tag: &'static str,
}

impl BlockTag {
    fn add_block(&self) -> String {
        match self.block_type {
            "ordered" | "bullet" => format!("<{}>", self.tag),
            "code-block" => format!("<{} class=\"ql-syntax\" spellcheck=\"false\">", self.tag),
            _ => String::from("")
        }
    }

    fn add_item(&self, content: &String) -> String {
        match self.block_type {
            "ordered" | "bullet" => format!("<li>{}</li>", content),
            "code-block" => format!("{}\n", content),
            _ => String::from("")
        }
    }

    
}

pub struct BlockState {
    block_tag: HashMap<&'static str, BlockTag>,
    current_block: Option<BlockTag>,
}

impl BlockState {
    pub fn new() -> BlockState {
        let mut block_tag: HashMap<&str, BlockTag> = HashMap::new();
        block_tag.insert("ordered", BlockTag{ block_type: "ordered", tag: "ol" });
        block_tag.insert("bullet", BlockTag{ block_type: "bullet", tag: "ul" });
        block_tag.insert("code-block", BlockTag{ block_type: "code-block", tag: "pre" });

        BlockState {
            block_tag: block_tag, 
            current_block: None
        }
    }
    pub fn open_block(&mut self, block_type: &String, content: &String) -> String {
        let mut pending = String::from("");
        
        if let Some(target_block) = self.block_tag.get(block_type.as_str()) {
            // we are in a list block
            if let Some(current_block) = &self.current_block  {
                // block type not change, just pend block item into it
                if current_block.block_type == target_block.block_type {
                    pending = current_block.add_item(content);
                }else{
                    // wo get a new block with different type, need close the last block first
                    let result = format!("</{}>{}{}", current_block.tag, target_block.add_block(), target_block.add_item(content) );
                    self.current_block = Some(target_block.clone());

                    pending = result;
                }
            }else{
                // a totally new list block
                let result = format!("{}{}", target_block.add_block(), target_block.add_item(content));
                self.current_block = Some(target_block.clone());

                pending = result;
            }
        }
        pending
    }


    pub fn check_and_close_current_block(&mut self) -> String {
        let mut pending = String::from("");
        if let Some(current_block) = &self.current_block  {
            pending = format!("</{}>", current_block.tag);
            self.current_block = None;
        }
        pending
    }
}