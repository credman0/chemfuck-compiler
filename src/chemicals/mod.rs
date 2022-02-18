#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct ChemToken {
    pub quantity:u32,
    pub chemical:Chemical,
    pub priority:u32
}

impl ChemToken {
    // pub fn add(&self, other:&ChemToken) -> ChemToken {
    //     for i in 0..self.chemical.chemicals.len() {
    //         let chemical = self.chemical.chemicals[i];
    //         if chemical.chemical == other.chemical {
    //             let old = self.chemical.chemicals.remove(i);
    //             let new = old
    //             self.chemical.chemicals.insert(i, old);
    //             return;
    //         }
    //     }
    //     // not found
    //     self.chemical.chemicals.push(other.clone());
    // }

    pub fn size(&self) -> u32 {
        let mut sum = self.quantity;
        for chemical in &self.chemical.chemicals {
            sum += chemical.size();
        }
        return sum;
    }

    pub fn combine (&mut self, other:&ChemToken) {
        if !self.combinable(other) {
            panic!();
        }
        if self.chemical.name.is_some() && other.chemical.name.is_some() {
            self.quantity += other.quantity;
        }
        for chem in &other.chemical.chemicals {
            for self_chem in &mut self.chemical.chemicals {
                if self_chem.combinable(chem) {
                    self_chem.combine(chem);
                    break;
                }
            }
        }
    }

    pub fn combinable (&self, other:&ChemToken) -> bool{
        if self.chemical.name.is_some() && other.chemical.name.is_some() {
            return self.chemical.name.as_ref().unwrap() == other.chemical.name.as_ref().unwrap();
        }
        for chem in &self.chemical.chemicals {
            if !other.chemical.chemicals.contains(chem) {
                return false;
            }
        }
        for chem in &other.chemical.chemicals {
            if !self.chemical.chemicals.contains(chem) {
                return false;
            }
        }
        return true;
    }
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct Chemical {
    pub name:Option<String>,
    pub chemicals:Vec<ChemToken>,
    pub temp:Option<u32>
}