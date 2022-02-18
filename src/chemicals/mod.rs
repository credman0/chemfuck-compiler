#[derive(Debug, Clone, Default, Hash, Eq)]
pub struct ChemToken {
    pub quantity:NumberToken,
    pub chemical:Chemical,
    pub priority:u32,
    pub concrete_quantity:Option<u32>
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum NumberToken {
    Constant(u32),
    Calculated(NumberOperator)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NumberOperator {
    numerator:u32,
    denominator:u32
}

impl NumberOperator {
    pub fn new (numerator:u32,denominator:u32) -> NumberOperator {
        return NumberOperator {numerator, denominator};
    }
}

pub fn div_up(a: u32, b: u32) -> u32 {
    (a + (b - 1))/b
}

impl NumberToken {
    fn calculate(&self, parent_value:Option<u32>) -> u32 {
        match self {
            NumberToken::Constant (val) => {
                return *val;
            },
            NumberToken::Calculated (operator) => {
                let parent_value = parent_value.unwrap();
                return div_up(parent_value*operator.numerator, operator.denominator);
            }
        }
    }
    
    pub fn as_text(&self) -> String {
        match self {
            NumberToken::Constant(val) => {
                return format!("{}", val);
            },
            NumberToken::Calculated(operator) => {
                return format!("${}/{}", operator.numerator, operator.denominator);
            }
        }
    }

    pub fn is_constant(&self) -> bool {
        match self {
            NumberToken::Constant (_) => {return true}
            _ => return false
        }
    }
}

impl Default for NumberToken {
    fn default() -> Self { NumberToken::Constant(0) }
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

    pub fn combine (&mut self, other:&ChemToken) {
        if !self.combinable(other) {
            panic!();
        }
        match &self.quantity {
            NumberToken::Constant(val) => {
                match other.quantity {
                    NumberToken::Calculated(_) => {
                        panic!("tried to add calculated to constant");
                    },
                    NumberToken::Constant(other_val) => {
                        self.quantity = NumberToken::Constant(val+other_val);
                    }
                }
            },
            NumberToken::Calculated(operator) => {
                match &other.quantity {
                    NumberToken::Calculated(other_operator) => {
                        assert_eq!(other_operator.denominator, operator.denominator);
                        let new_numerator = operator.numerator + other_operator.numerator;
                        self.quantity = NumberToken::Calculated(NumberOperator{numerator:new_numerator, denominator:operator.denominator});
                    },
                    NumberToken::Constant(_) => {
                        panic!("tried to add constant to calculated");
                    }
                    
                }
            }
        }
    }

    pub fn size(&self) -> u32 {
        let mut sum = 0;
        if self.chemical.chemicals.is_empty() {
            sum += self.concrete_quantity.unwrap();
        } else {
            for chemical in &self.chemical.chemicals {
                sum += chemical.concrete_quantity.unwrap();
            }
        }
        return sum;
    }

    pub fn combinable (&self, other:&ChemToken) -> bool{
        return self.chemical == other.chemical && self.quantity.is_constant() == other.quantity.is_constant();
    }

    pub fn set_concrete_quantity(&mut self, parent_quantity:u32) {
        match &self.quantity {
            NumberToken::Constant(val)=>  {
                self.concrete_quantity = Some(*val);
            },
            NumberToken::Calculated(_) => {
                self.concrete_quantity = Some(self.quantity.calculate(Some(parent_quantity)));
            }
        }
        for chem in &mut self.chemical.chemicals {
            chem.set_concrete_quantity(self.concrete_quantity.unwrap());
        }
    }

    /// for error checking, if modifying the concrete value unset the childrens' concrete values
    pub fn set_children_abstract(&mut self) {
        for child in &mut self.chemical.chemicals {
            child.concrete_quantity = None;
        }
    }
}

#[derive(Debug, Clone, Default, Hash, Eq)]
pub struct Chemical {
    pub name:Option<String>,
    pub chemicals:Vec<ChemToken>,
    pub temp:Option<u32>
}

impl PartialEq for Chemical {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.chemicals == other.chemicals
    }
}

impl PartialEq for ChemToken {
    fn eq(&self, other: &Self) -> bool {
        self.quantity == other.quantity && self.chemical == other.chemical
    }
}