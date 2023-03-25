use std::collections::HashMap;

struct Directory {
    name: String,
    files: HashMap<String, File>,
    subdirectories: HashMap<String, Directory>,
    parent: Option<Box<Directory>>,
    read_permission: bool,
    write_permission: bool,
}

impl Directory {
    fn new(name: &str, parent: Option<Box<Directory>>) -> Self {
        Directory {
            name: name.to_owned(),
            files: HashMap::new(),
            subdirectories: HashMap::new(),
            parent: parent,
            read_permission: true,
            write_permission: true,
        }
    }
    
    fn add_file(&mut self, name: &str, content: &str, read_permission: bool, write_permission: bool) {
        self.files.insert(name.to_owned(), File::new(name, content, read_permission, write_permission));
    }
    
    fn add_directory(&mut self, name: &str, read_permission: bool, write_permission: bool) -> &mut Directory {
        self.subdirectories.entry(name.to_owned())
            .or_insert_with(|| Directory::new(name, Some(Box::new(self.clone()))))
            .read_permission = read_permission;
        self.subdirectories.get_mut(name).unwrap().write_permission = write_permission;
        self.subdirectories.get_mut(name).unwrap()
    }

    fn delete_file(&mut self, name: &str) {
        self.files.remove(name);
    }

    fn print_directory_contents(&self) {
        let mut contents: Vec<&String> = self.files.keys().collect();
        contents.append(&mut self.subdirectories.keys().collect());
        contents.sort();

        println!("{} contents:", self.name);
        println!("{} files and {} directories.", self.files.len(), self.subdirectories.len());

        for item in contents {
            let size = match self.files.get(item) {
                Some(file) => file.content.len(),
                None => 0
            };
            let file_type = match self.files.get(item) {
                Some(_) => "File",
                None => "Directory"
            };
            println!("{} ({}) - {} bytes", item, file_type, size);
        }
    }
}

struct File {
    name: String,
    content: String,
    parent: Option<Box<Directory>>,
    read_permission: bool,
    write_permission: bool,
}

impl File {
    fn new(name: &str, content: &str, read_permission: bool, write_permission: bool) -> Self {
        File {
            name: name.to_owned(),
            content: content.to_owned(),
            parent: None,
            read_permission: read_permission,
            write_permission: write_permission,
        }
    }
    
    fn set_parent(&mut self, parent: &mut Directory) {
        self.parent = Some(Box::new(parent.clone()));
    }
}
