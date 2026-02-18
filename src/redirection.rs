use std::fs::OpenOptions;
#[derive(Debug)]

pub struct Redirection {
    pub options: OpenOptions,
    pub filename: Option<String>,
}
impl Redirection {
     fn new(overwrite: bool, filename: Option<String>) -> Redirection {
        let file_options = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(overwrite)
            .append(!overwrite)
            .clone();

        Redirection {
            options: file_options,
            filename,
        }
    }
   pub fn parse_redirections(tokens: &mut Vec<String>) -> (Option<Redirection>, Option<Redirection>) {
        let mut std_out_file = None;
        let mut std_err_file = None;
        while let Some(index) = tokens.iter().position(|t| {
            t == ">" || t == "1>" || t == ">>" || t == "1>>" || t == "2>" || t == "2>>"
        }) {
            let mut file_name = None;
            let operator = tokens.remove(index);
            match index + 1 <= tokens.len() {
                true => file_name = Some(tokens.remove(index)),
                false => (),
            }

            match operator.as_str() {
                ">" | "1>" => {
                    std_out_file = Some(Redirection::new(true, file_name));
                }
                ">>" | "1>>" => {
                    std_out_file = Some(Redirection::new(false, file_name));
                }
                "2>" => {
                    std_err_file = Some(Redirection::new(true, file_name));
                }
                "2>>" => {
                    std_err_file = Some(Redirection::new(false, file_name));
                }
                _ => {}
            }
        }

        (std_out_file, std_err_file)
    }
}