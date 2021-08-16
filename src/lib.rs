use std::{ collections::HashMap };
use serde_json::{ Value };
use serde::{ Deserialize, Serialize };



#[derive(Deserialize, Serialize)]
pub struct DeltaOp {
    insert: Value,
    attributes: Option<Value>
}

struct ListBlockTag {
    list_type: &'static str,
    start: &'static str,
    close: &'static str
}

pub fn parser(delta_ops: Vec<DeltaOp>) -> String {
    let mut list_block_tag: HashMap<&str, ListBlockTag> = HashMap::new();
    list_block_tag.insert("ordered", ListBlockTag{ list_type: "ordered", start: "<ol>", close: "</ol>" });
    list_block_tag.insert("bullet", ListBlockTag{ list_type: "bullet", start: "<ul>", close: "</ul>" });
    
    let mut html = String::from("");
        
    let mut list_block:Option<&ListBlockTag> = None;
    let mut reader = String::from("");

    for op in delta_ops.iter() {
        if let Value::String(str_insert) = &op.insert {
            let mut inner_reader = String::from("");
            for (_, char) in str_insert.char_indices() {
                if char != '\n' {
                    inner_reader.push(char);
                } else {
                    reader.push_str(&inner_reader);
                    inner_reader.clear();

                    let mut pending = String::from("");
                    if let Some(Value::Object(attr)) = &op.attributes {
                        if let Some(Value::String(next_list_type))  = attr.get("list") {
                            if let Some(current_list_block) = list_block {
                                if current_list_block.list_type == next_list_type {
                                    pending = format!("<li>{}</li>", &reader);
                                }else{
                                    let tmp_list_block = list_block_tag.get(next_list_type.as_str()).unwrap();

                                    let result = format!("{}{}<li>{}</li>", current_list_block.close, tmp_list_block.start, &reader);
                                    list_block = Some(tmp_list_block);

                                    pending = result;
                                }
                            }else{
                                let tmp_list_block = list_block_tag.get(next_list_type.as_str()).unwrap();

                                let result = format!("{}<li>{}</li>", tmp_list_block.start, &reader);
                                list_block = Some(tmp_list_block);

                                pending = result;
                            }
                        }else if let Some(Value::Number(header)) = attr.get("header") {
                            let tmp_content = if reader.is_empty() {
                                "</br>"
                            }else{
                                reader.as_str()
                            };
                            // rust
                            let mut result = format!("<h{}>{}</h{}>", header, tmp_content, header);

                            if let Some(current_list_block) = list_block {
                                result = format!("{}{}", current_list_block.close, &result);

                                list_block = None;
                            }
                            pending = result;
                        }
                    } else {
                        let tmp_content = if reader.is_empty() {
                            "</br>"
                        }else{
                            reader.as_str()
                        };
                        let mut result =  format!("<p>{}</p>", tmp_content);

                        if let Some(current_list_block) = list_block {
                            result = format!("{}{}", current_list_block.close, &result);

                            list_block = None;
                        }

                        pending = result;
                    }


                    html.push_str(&pending);
                    reader.clear();
                }
            }
                
            if !inner_reader.is_empty() {
                reader.push_str(format(inner_reader, &op.attributes).as_str());
            }
        } else if let Value::Object(obj_insert) = &op.insert {
            if let Some(Value::String(savvy_image)) = obj_insert.get("savvy_image") {
                let tmp_alt = match &op.attributes {
                    Some(Value::Object(attr)) => {
                        attr.get("alt").and_then(|v| v.as_str()).unwrap_or_default()
                    }
                    _ => ""
                };

                reader.push_str(&format!("<img src=\"{}\" alt=\"{}\"/>", savvy_image, tmp_alt));
            } else if let Some(Value::Object(mention)) = obj_insert.get("mention") {
                let mention_index = mention.get("index").and_then(|v|v.as_u64()).unwrap_or_default();
                let mention_id = mention.get("id").and_then(|v|v.as_u64()).unwrap_or_default();
                let mention_value = mention.get("value").and_then(|v|v.as_str()).unwrap_or_default();

                reader.push_str(&format!("<span class=\"mention\" data-index=\"{}\" data-denotation-char=\"@\" data-id=\"{}\" data-value=\"{}\">&#xFEFF;<span contenteditable=\"false\"><span class=\"ql-mention-denotation-char\">@</span>{}</span>&#xFEFF;</span>", mention_index, mention_id, mention_value, mention_value));
            }
        }
    }


    if let Some(current_list_block) = list_block {
        html.push_str(current_list_block.close);
    }

    html
}


fn format(mut raw_input: String, attr: &Option<Value>) -> String {
    if let Some(Value::Object(inner_attr)) = attr {
        for(key, value) in inner_attr {
            match key.as_str() {
                "link" => {
                    let href = value.as_str().unwrap_or_default();
                    raw_input = format!("<a href=\"{}\" rel=\"noopener noreferrer\" target=\"_blank\" title=\"{}\">{}</a>", href, href, raw_input); 
                }
                "underline" => { raw_input = format!("<u>{}</u>", raw_input); }
                "strike" => { raw_input = format!("<s>{}</s>", raw_input); }
                "italic" => { raw_input = format!("<em>{}</em>", raw_input); }
                "bold" => { raw_input = format!("<strong>{}</strong>", raw_input); }
                
                _ => ()
            }
        }
    }
    raw_input
}



#[cfg(test)]
mod tests {
    use crate::parser;
    use crate::DeltaOp;
    use serde_json::json;
    use serde_json::{ Value };

    #[test]
    fn test_base() {
        let result = parser(vec![DeltaOp {insert: Value::String(String::from("hello world\n")), attributes: None}]);
        assert_eq!(result, String::from("<p>hello world</p>"));
    }

    #[test]
    fn test_with_attr() {
        let result = parser(vec![DeltaOp {
            insert: Value::String(String::from("hello world")), 
            attributes: Some(json!({"underline": true, "strike": true, "italic": true, "bold": true, "link": "https://www.test.com"}))
        }, DeltaOp {
            insert: Value::String(String::from("\n")),
            attributes: None
        }]);
        assert_eq!(result, String::from("<p><u><s><a href=\"https://www.test.com\" rel=\"noopener noreferrer\" target=\"_blank\" title=\"https://www.test.com\"><em><strong>hello world</strong></em></a></s></u></p>"));
    }

    #[test]
    fn test_with_list_1() {
        let result = parser(vec![DeltaOp {
            insert: Value::String(String::from("1")), 
            attributes: None
        }, DeltaOp {
            insert: Value::String(String::from("23")), 
            attributes: Some(json!({"bold": true}))
        }, DeltaOp {
            insert: Value::String(String::from("\n")),
            attributes: Some(json!({"list": "ordered"}))
        }, DeltaOp {
            insert: Value::String(String::from("abc")), 
            attributes: Some(json!({"italic": true, "link": "abc", "strike": true, "underline": true}))
        }, DeltaOp {
            insert: Value::String(String::from("➗")), 
            attributes: Some(json!({"link": "abc"}))
        }, DeltaOp {
            insert: Value::String(String::from("\n")),
            attributes: Some(json!({"list": "ordered"}))
        }]);

        assert_eq!(result, String::from("<ol><li>1<strong>23</strong></li><li><u><s><a href=\"abc\" rel=\"noopener noreferrer\" target=\"_blank\" title=\"abc\"><em>abc</em></a></s></u><a href=\"abc\" rel=\"noopener noreferrer\" target=\"_blank\" title=\"abc\">➗</a></li></ol>"));

    }
}