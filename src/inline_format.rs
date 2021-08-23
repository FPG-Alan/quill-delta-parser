use serde_json::{ Value };


// inline format
struct FormatTag {
    key: &'static str,
    tag: &'static str,
    context: Option<String>
}

impl FormatTag {
    fn format(&self, input: String, style_input: Option<String>) -> String {
        if self.key == "a" {
            let defaut_href = String::from("");
            let href = self.context.as_ref().unwrap_or(&defaut_href);

            if style_input != None {
                format!("<a href=\"{}\" rel=\"noopener noreferrer\" target=\"_blank\" title=\"{}\" style=\"{}\">{}</a>", href, href, style_input.unwrap_or_default(), input)
            } else {
                format!("<a href=\"{}\" rel=\"noopener noreferrer\" target=\"_blank\" title=\"{}\">{}</a>", href, href, input)
            }
        } else {
            if style_input != None { 
                format!("<{} style=\"{}\">{}</{}>", self.tag, style_input.unwrap_or_default(), input, self.tag)
            }else{
                format!("<{}>{}</{}>", self.tag, input, self.tag)

            }
        }
    }
}
pub fn format(mut raw_input: String, attr: &Option<Value>) -> String {
    if let Some(Value::Object(inner_attr)) = attr {
        let mut styled_attrs_str = String::from("");
        let mut formatters:Vec<FormatTag> = Vec::new();
        for(key, value) in inner_attr {
            match key.as_str() {
                "link" => {
                    formatters.push(FormatTag{ key: "a", tag: "a", context:Some(String::from(value.as_str().unwrap_or_default())) });
                }
                "underline" => { 
                    formatters.push(FormatTag{ key: "underline", tag: "u", context:None });
                }
                "strike" => { 
                    formatters.push(FormatTag{ key: "strike", tag: "s", context:None });

                 }
                "italic" => { 
                    formatters.push(FormatTag{ key: "italic", tag: "em", context:None });
                    
                 }
                "bold" => { 
                    formatters.push(FormatTag{ key: "bold", tag: "strong", context:None });

                 }
                "code" => { 
                    formatters.push(FormatTag{ key: "code", tag: "code", context:None });
                 }

                
                "color" => { styled_attrs_str.push_str(&format!("color: {}; ", value.as_str().unwrap_or_default())); }
                "background" => { styled_attrs_str.push_str(&format!("background-color: {}; ", value.as_str().unwrap_or_default())); }
                _ => ()
            }
        }
        if formatters.len() == 0 {
            formatters.push(FormatTag { key: "inline", tag: "span", context:None })
        }

        for (index, item) in formatters.iter().enumerate() {
            // the last one
            if index == formatters.len() - 1 && !styled_attrs_str.is_empty() {
                raw_input = item.format(raw_input, Some(styled_attrs_str.clone()));
            } else {
                raw_input = item.format(raw_input, None);
            }
        }


    }
    raw_input
}
