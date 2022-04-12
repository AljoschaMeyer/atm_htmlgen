#[derive(Clone, Debug)]
pub struct HSections {
    structure: HStructure,
    finished_structure: HStructure,
    current_path: Vec<usize>,
}

impl HSections {
    pub fn new() -> Self {
        HSections {
            structure: HStructure {
                id: "".to_string(),
                children: Vec::new(),
            },
            finished_structure: HStructure {
                id: "".to_string(),
                children: Vec::new(),
            },
            current_path: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_path = Vec::new();
        self.finished_structure = self.structure.clone();
        self.structure = HStructure {
            id: "".to_string(),
            children: Vec::new(),
        };
    }

    pub fn push(&mut self, id: String, second_iteration: bool) {
        let current_hstructure = self.structure.path_to_current(&self.current_path);
        self.current_path.push(current_hstructure.children.len());

        current_hstructure.children.push(HStructure {
            id: id.clone(),
            children: Vec::new(),
        });
    }

    pub fn pop(&mut self) {
        self.current_path.pop();
    }

    pub fn previous_and_next_ids(&self) -> (Option<&str>, Option<&str>) {
        let path_previous = self.finished_structure.path_to_previous_sibling(&self.current_path);
        let path_next = self.finished_structure.path_to_next_sibling(&self.current_path);
        return (path_previous, path_next);
    }
}

#[derive(Clone, Debug)]
struct HStructure {
    id: String,
    children: Vec<HStructure>,
}

impl HStructure {
    fn path_to_current(&mut self, p: &[usize]) -> &mut Self {
        if p.len() == 0 {
            return self;
        } else {
            return self.children[p[0]].path_to_current(&p[1..]);
        }
    }

    fn path_to_previous_sibling(&self, p: &[usize]) -> Option<&str> {
        if p.len() == 0 {
            return Some(&self.id);
        } else {
            let i = if p.len() == 1 {
                p[0].checked_sub(1)?
            } else {
                p[0]
            };

            return self.children.get(i)?.path_to_previous_sibling(&p[1..]);
        }
    }

    fn path_to_next_sibling(&self, p: &[usize]) -> Option<&str> {
        if p.len() == 0 {
            return Some(&self.id);
        } else {
            let i = if p.len() == 1 {
                p[0] + 1
            } else {
                p[0]
            };

            return self.children.get(i)?.path_to_previous_sibling(&p[1..]);
        }
    }
}
