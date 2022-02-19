use crate::{Chemical, ChemToken, NumberToken};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU32;

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct ActionChain {
    intial_state:ChemState,
    chain:Vec<Action>
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct AbstractActionChain {
    chain:Vec<AbstractAction>
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Action {
    Transfer{amount:u32, source:u32, target:u32},
    Heat{temp:u32, target:u32},
    Eject{target:u32},
    DumpByproduct{target:u32, remaining:u32},
    EjectDownTo{target:u32, amount:u32},
    CreateBottle{target:u32, amount:u32},
    CreatePill{target:u32, amount:u32}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AbstractAction {
    Combine{vec:Vec<ChemToken>, temp:Option<u32>}
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct ChemState {
    chems:Vec<Reservoir>
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Reservoir {
    contents:Option<ChemToken>,
    reservoir_size:ReservoirSize
}

const NUM_RESERVOIRS:u32 = 10;
const SMALL_RESERVOIR:u32 = 50;
const LARGE_RESERVOIR:u32 = 100;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum ReservoirSize {
    Empty,
    Small,
    Large
}

impl ReservoirSize {
    fn get_size(&self) -> u32{
        match self {
            ReservoirSize::Empty => 0,
            ReservoirSize::Small => SMALL_RESERVOIR,
            ReservoirSize::Large => LARGE_RESERVOIR
        }
    }

    fn fit(size:u32) -> ReservoirSize {
        if size == 0 {
            return ReservoirSize::Empty;
        } else if size <= 50 {
            return ReservoirSize::Small;
        } else if size <= 100 {
            return ReservoirSize::Large;
        } else {
            panic!("No reservoir large enough to fit")
        }
    }
}

impl Reservoir {
    pub fn add(&mut self, chem:&ChemToken) {

    }

    pub fn new(contents:&ChemToken) -> Reservoir {
        Reservoir {contents:Some(contents.clone()), reservoir_size:ReservoirSize::fit(contents.size())}
    }

    pub fn empty() -> Reservoir {
        Reservoir {contents:None, reservoir_size:ReservoirSize::Empty}
    }

    pub fn upgrade_size(&mut self) -> Result<(), String> {
        match self.reservoir_size {
            ReservoirSize::Empty => {
                self.reservoir_size = ReservoirSize::Small;
                return Ok(());
            },
            ReservoirSize::Small => {
                self.reservoir_size = ReservoirSize::Large;
                return Ok(());
            },
            ReservoirSize::Large => return Err("Tried to upgrade large reservoir: Insufficient space!".to_string())
        }
    }

    pub fn replace(&mut self, chem:&ChemToken) -> Result<(), String> {
        if self.reservoir_size.get_size() < chem.size() {
            self.reservoir_size = ReservoirSize::fit(chem.size());
        }
        self.contents = Some(chem.clone());
        return Ok(());
    }

    pub fn clear(&mut self) {
        self.contents = None;
    }

    pub fn reduce(&mut self, amount:u32) {
        *self.contents.as_mut().unwrap().concrete_quantity.as_mut().unwrap() -= amount;
        self.contents.as_mut().unwrap().set_children_abstract();
        // if self.contents.as_ref().unwrap().quantity == 0 {
        //     self.clear();
        // }
    }

    pub fn multiply(&mut self, amount:u32) {
        if self.contents.is_none() {
            return;
        }
        *self.contents.as_mut().unwrap().concrete_quantity.as_mut().unwrap()*=amount;
        self.contents.as_mut().unwrap().quantity = NumberToken::Constant(self.contents.as_ref().unwrap().concrete_quantity.unwrap());
        if self.reservoir_size.get_size() < self.contents.as_ref().unwrap().size() {
            self.reservoir_size = ReservoirSize::fit(self.contents.as_ref().unwrap().size());
        }
    }
}

impl ChemState {
    pub fn multiply(&mut self, amount:u32) {
        for reservoir in &mut self.chems {
            reservoir.multiply(amount);
        }
    }

    pub fn to_text(&self) -> String{
        let mut text = "[".to_string();
        for i in 0..self.chems.len() {
            let name:String = if self.chems[i].contents.is_some() {self.chems[i].contents.as_ref().unwrap().chemical.name.as_ref().unwrap_or(&"None".to_string()).to_string()} else {"None".to_string()};
            let quantity:NumberToken = if self.chems[i].contents.is_some() {self.chems[i].contents.as_ref().unwrap().quantity.clone()} else {NumberToken::default()};
            text = format!("{},  r{}:({}) {}", text, i+1,quantity.as_text(), name, );
        }
        return format!("{}]",text);
    }

    pub fn size(&self) -> u32 {
        let mut sum = 0;
        for reservoir in &self.chems {
            if reservoir.contents.is_some() {
                sum += reservoir.contents.as_ref().unwrap().size();
            }
        }
        return sum;
    }

    pub fn find_chem(&self, chem:&Chemical) -> Option<usize> {
        for i in 0..self.chems.len() {
            let reschem = &self.chems[i].contents;
            if reschem.is_some() {
                if &reschem.as_ref().unwrap().chemical == chem {
                    return Some(i);
                }
            }
        }
        return None;
    }

    pub fn new(reservoirs:&Vec<Reservoir>) -> ChemState {
        let mut self_reservoirs = reservoirs.clone();
        while self_reservoirs.len() < NUM_RESERVOIRS as usize {
            self_reservoirs.push(Reservoir::empty());
        }
        return ChemState {chems:self_reservoirs};
    }

    pub fn get(&self, index:usize) -> Reservoir {
        return self.chems.get(index).unwrap().clone();
    }

    pub fn replace(&mut self, index:usize, chem:&ChemToken) {
        self.chems.get_mut(index).unwrap().replace(chem).unwrap();
    }

    pub fn clear(&mut self, index:usize) {
        self.chems.get_mut(index).unwrap().clear();
    }

    pub fn reduce(&mut self, index:usize, amount:u32) {
        self.chems.get_mut(index).unwrap().reduce(amount);
    }

    pub fn first_empty(&self) -> usize {
        for i in 0..self.chems.len() {
            let reservoir = &self.chems[i];
            if reservoir.contents.is_none() {
                return i;
            }
        }
        panic!("No empty reservoirs!");
    }

    pub fn count_nonempty(&self) -> u32 {
        let mut count = 0;
        for reservoir in &self.chems {
            if reservoir.contents.is_some() {
                count+=1;
            }
        }
        return count;
    }

    pub fn pad(&mut self) {
        self.chems.retain(|x| x.contents.is_some());
        while self.chems.len() < NUM_RESERVOIRS as usize {
            self.chems.push(Reservoir::empty());
        }
    }

    pub fn get_sizes(&self) -> Vec<u32> {
        let mut sizes = vec![];
        for reservoir in &self.chems {
            sizes.push(reservoir.reservoir_size.get_size());
        }
        return sizes;
    }
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct ChemTree {
    pub initial_state:ChemState,
    root:ChemTreeBranch
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
struct ChemTreeBranch {
    chem:ChemToken,
    children:Vec<ChemTreeBranch>,
    id:u32,
}

impl ChemTreeBranch {
    pub fn get_leaves(&self) -> Vec<ChemTreeBranch> {
        if self.children.is_empty() {
            return vec![self.clone()];
        }
        let mut leaves = vec![];
        self.get_leaves_rec(&mut leaves);
        return leaves;
    }

    fn get_leaves_rec(&self, leaves:&mut Vec<ChemTreeBranch>) {
        for child in &self.children {
            if child.children.is_empty() {
                leaves.push(child.clone());
            } else {
                child.get_leaves_rec(leaves);
            }
        }

    }

    pub fn remove_leaf(&mut self, branch:&ChemTreeBranch) -> bool {
        if !branch.is_leaf() {
            panic!("TRIED TO TRIM NON_LEAF");
        }
        let len = self.children.len();
        self.children.retain(|x| x.id!=branch.id);
        if len == self.children.len() { // nobody pruned
            for child in &mut self.children {
                let res = child.remove_leaf(branch);
                if res {
                    return true;
                }
            }
            return false;
        } else {
            return true;
        }
    }

    pub fn dissolve_branch(&mut self, branch:&ChemTreeBranch) -> bool {
        let mut found_branch = None;
        for child in  &self.children {
            if child.id == branch.id { // found it
                found_branch = Some(child.clone());
            }
        }
        if found_branch.is_some() {
            let child = found_branch.unwrap();
            for child_child in &child.children {
                self.children.push(child_child.clone());
            }
            self.children.retain(|x| x.id != child.id);
            return true;
        } else {
            for child in  &mut self.children {
                if child.dissolve_branch(&branch) {
                    return true;
                }
            }
            return false;
        }
    }

    pub fn get_branches_with_chem (&self, chem:&Chemical) -> Vec<ChemTreeBranch> {
        let mut branches = vec![];
        self.get_branches_with_chem_rec(chem, &mut branches);
        return branches;
    }

    fn get_branches_with_chem_rec(&self, chem:&Chemical, branches:&mut Vec<ChemTreeBranch>) {
        if &self.chem.chemical == chem {
            branches.push(self.clone());
        }
        for child in &self.children {
            child.get_branches_with_chem_rec(chem, branches);
        }
    }

    pub fn simplify(&mut self) {
        let mut all_chems = self.bucket_chems();
        println!("{:?}", all_chems);
        let mut mut_tree = self.clone();
        // trim the basic chems first
        let leaves = mut_tree.get_leaves();
        for leaf in &leaves {
            mut_tree.remove_leaf(leaf);
        }
        'main: while !mut_tree.children.is_empty() {
            let chem_keys:Vec<Chemical> = all_chems.keys().map(|x| x.clone()).collect();
            for chem in &chem_keys {
                let branches = self.get_branches_with_chem(chem);
                if branches.is_empty() {
                    panic!("No branches with chem");
                }
                let mut selected_leaf = None;
                for branch in &branches {
                    if branch.is_leaf() {
                        selected_leaf = Some(branch);
                        break;
                    }
                }
                if selected_leaf.is_some() {
                    let selected_leaf = selected_leaf.unwrap();
                    let mut new_chem_token = selected_leaf.chem.clone();
                    for branch in &branches {
                        if branch.id == selected_leaf.id {
                            continue;
                        }
                        new_chem_token.combine(&branch.chem);
                        self.dissolve_branch(branch);
                        mut_tree.dissolve_branch(branch);
                    }
                    let counted_amount = &all_chems[chem];
                    assert_eq!(counted_amount.concrete_quantity.unwrap(), new_chem_token.concrete_quantity.unwrap());
                    println!("Adding {:?} to {}", &new_chem_token, selected_leaf.id);
                    if !self.replace_chem_in_branch(&new_chem_token, selected_leaf.id) {
                        panic!("Missing leaf");
                    }
                    all_chems.remove(chem);
                    continue 'main;
                }
            }
            let leaves = mut_tree.get_leaves();
            for leaf in &leaves {
                mut_tree.remove_leaf(leaf);
            }
        }
        if !all_chems.is_empty() {
            println!("Missing chems: {:?}", all_chems);
        }
    } 

    fn bucket_chems(&self) -> HashMap<Chemical, ChemToken> {
        let mut buckets = HashMap::new();
        self.bucket_chems_recursive(&mut buckets);
        return buckets;
    }

    fn bucket_chems_recursive(&self,buckets:&mut HashMap<Chemical, ChemToken>) {
        if !buckets.contains_key(&self.chem.chemical) {
            buckets.insert(self.chem.chemical.clone(), self.chem.clone());
        }
        let mut new_chem = self.chem.clone();
        new_chem.combine(&buckets[&self.chem.chemical]);
        buckets.insert(self.chem.chemical.clone(), new_chem);
        for child in &self.children {
            child.bucket_chems_recursive(buckets);
        }
    }

    pub fn is_leaf(&self) -> bool {
        return self.children.is_empty();
    }

    pub fn replace_chem_in_branch(&mut self, chem:&ChemToken, branch_id:u32) -> bool {
        if self.id == branch_id {
            self.chem = chem.clone();
            return true;
        }
        for child in &mut self.children {
            let result = child.replace_chem_in_branch(chem, branch_id);
            if result {
                return true;
            }
        }
        return false;
    }

    pub fn concretize_quantites (&mut self) {
        if self.chem.quantity.is_constant() {
            self.chem.set_concrete_quantity(0);
        }
        for child in &mut self.children {
            child.chem.set_concrete_quantity(self.chem.concrete_quantity.unwrap());
            child.concretize_quantites();
        }
    }
}


impl ChemTreeBranch {
    fn deconstruct(token:&ChemToken, id_counter:&mut AtomicU32) -> ChemTreeBranch {
        let mut children = vec![];
        let chem = token;
        for child in &chem.chemical.chemicals {
            children.push(ChemTreeBranch::deconstruct(child, id_counter));
        }
        return ChemTreeBranch {chem:chem.clone(), children, id:id_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed)};
    }
}

impl ChemTree {
    pub fn deconstruct(token:&ChemToken) -> ChemTree {
        let mut root = ChemTreeBranch::deconstruct(token, &mut AtomicU32::new(0));
        root.concretize_quantites();
        let initial_state = compute_initial_state(&root.chem);
        return ChemTree {root, initial_state};
    }
}

pub fn compute_initial_state (final_chem:&ChemToken) -> ChemState {
    let mut chem_map = HashMap::new();
    count_raw_chems_recursive(&mut chem_map, final_chem);
    let mut temps_map = HashSet::new();
    get_temps_recursive(&mut temps_map, final_chem);
    let mut chems_vec = vec![];
    for chem in chem_map.keys() {
        chems_vec.push(ChemToken{quantity:NumberToken::Constant(*chem_map.get(chem).unwrap()), chemical:chem.clone(), concrete_quantity:Some(*chem_map.get(chem).unwrap()), ..Default::default()});
    }   
    return ChemState::new(&chems_vec.into_iter().map(|x| Reservoir::new(&x)).collect());
}

fn count_raw_chems_recursive(chem_map:&mut HashMap<Chemical, u32>, chem:&ChemToken) {
    if chem.chemical.name.is_some() {
        if !chem_map.contains_key(&chem.chemical) {
            chem_map.insert(chem.chemical.clone(), 0);
        }
        chem_map.insert(chem.chemical.clone(), chem_map.get(&chem.chemical).unwrap() + chem.concrete_quantity.unwrap());
    }
    for next_chem in &chem.chemical.chemicals {
        count_raw_chems_recursive(chem_map, next_chem);
    }
}

fn get_temps_recursive(temps_map:&mut HashSet<u32>, chem:&ChemToken) {
    if chem.chemical.temp.is_some() {
        temps_map.insert(chem.chemical.temp.unwrap());
    }
    for next_chem in &chem.chemical.chemicals {
        get_temps_recursive(temps_map, next_chem);
    }
}

fn compress_state(state:&mut ChemState, tree:&mut ChemTree) {

}

pub fn compute_actions(tree:&ChemTree, times_produced:u32) -> (Vec<Action>, Vec<u32>) {
    let mut state = tree.initial_state.clone();
    while state.chems.len() > NUM_RESERVOIRS as usize {
        println!("TOO MANY CHEMICALS");
        panic!();
    }

    let allowed_mix_reservoirs_min_index;
    if times_produced > 1 {
        allowed_mix_reservoirs_min_index = find_intially_empty(&state);
    } else {
        allowed_mix_reservoirs_min_index = 0;
    }
    let mut actions = vec![];
    for _ in 0..times_produced {
        let mut mut_tree = tree.root.clone();
        // mut_tree.simplify();
        trim_basics(&mut mut_tree);
        while !mut_tree.children.is_empty() {
            compute_step(&mut state, &mut mut_tree, &mut actions, allowed_mix_reservoirs_min_index);
        }
        compute_step(&mut state, &mut mut_tree, &mut actions, allowed_mix_reservoirs_min_index); // final mix step
        let output_chem = &tree.root.chem;
        let output_reservoir_index = state.find_chem(&output_chem.chemical).unwrap();
        actions.push(Action::CreateBottle{target:output_reservoir_index as u32 + 1, amount:100});
        state.clear(output_reservoir_index);
        for i in allowed_mix_reservoirs_min_index..NUM_RESERVOIRS {
            state.clear(i as usize);
        }
    }
    // if mut_tree.chem.chemical.temp.is_some() {
    //     println!("\n\nSTATE: {:?}\n CHEM: {:?}", state, &mut_tree.chem.chemical);
    //     let reservoir_index = state.find_chem(&mut_tree.chem.chemical).unwrap();
    //     actions.push(Action::Heat{target:reservoir_index as u32, temp:mut_tree.chem.chemical.temp.unwrap()})
    // }
    
    return (actions,state.get_sizes());
}

fn find_intially_empty(initial_state:&ChemState) -> u32 {
    return initial_state.first_empty() as u32;
}

fn trim_basics(tree:&mut ChemTreeBranch) {
    let leaves = tree.get_leaves();
    for leaf in &leaves {
        tree.remove_leaf(leaf);
    }
}

fn emptied_chemicals(chem:&ChemToken, state:&ChemState, allowed_mix_reservoirs_min_index:u32) -> u32 {
    let mut emptied_count = 0;
    for chemical in &chem.chemical.chemicals {
        for reservoir in &state.chems {
            if reservoir.contents.is_some(){
                let reservoir_chemical = reservoir.contents.as_ref().unwrap();
                if reservoir_chemical.chemical == chemical.chemical {
                    if reservoir_chemical.concrete_quantity.unwrap() - chemical.concrete_quantity.unwrap() == 0 {
                        emptied_count += 1;
                    }
                }
            }
        }
    }
    return emptied_count;
}

fn compute_step(state:&mut ChemState, tree:&mut ChemTreeBranch, actions:&mut Vec<Action>, allowed_mix_reservoirs_min_index:u32) -> u32{
    let mut leaves = tree.get_leaves();
    leaves.sort_by(|x1,x2| {
        x2.chem.priority.cmp(&x1.chem.priority)
    });
    let max_priority = leaves.first().unwrap().chem.priority;
    leaves.retain(|x| x.chem.priority == max_priority);
    let mut picked = leaves.first().unwrap();
    let mut max_emptied = emptied_chemicals(&picked.chem, state, allowed_mix_reservoirs_min_index);
    for i in 1..leaves.len() {
        let trial = leaves.get(i).unwrap();
        let emptied = emptied_chemicals(&trial.chem,state, allowed_mix_reservoirs_min_index);
        if emptied > max_emptied {
            picked = trial;
            max_emptied = emptied;
        }
    }
    tree.remove_leaf(picked);
    let mut chems = picked.chem.chemical.chemicals.clone();
    chems.sort_by(|x1, x2| x2.priority.cmp(&x1.priority));
    // remove before finding an empty reservoir in case one of them opens up
    for chem in &chems {
        let reservoir_index = state.find_chem(&chem.chemical).unwrap();
        state.reduce(reservoir_index, chem.concrete_quantity.unwrap());
    }
    let mut combine_reservoir = None;
    for chem in &chems {
        let reservoir_index = state.find_chem(&chem.chemical).unwrap();
        let reservoir = state.get(reservoir_index);
        if reservoir.contents.as_ref().unwrap().concrete_quantity.unwrap() == 0 && reservoir_index as u32 >= allowed_mix_reservoirs_min_index {
            if combine_reservoir.is_none() {
                actions.push(Action::EjectDownTo{amount:chem.concrete_quantity.unwrap(), target:reservoir_index as u32 + 1});
                combine_reservoir = Some(reservoir_index);
            }
        }
    }

    if combine_reservoir.is_none() {
        combine_reservoir = Some(state.first_empty());
    }   
    let combine_reservoir = combine_reservoir.unwrap();

    for chem in &chems {
        let reservoir_index = state.find_chem(&chem.chemical).unwrap();
        if reservoir_index != combine_reservoir {
            actions.push(Action::Transfer{amount:chem.concrete_quantity.unwrap(), target:combine_reservoir as u32 + 1, source:reservoir_index as u32 + 1});
            if chem.concrete_quantity.unwrap() == 0 {
                actions.push(Action::Eject{target:reservoir_index as u32 + 1});
                state.clear(reservoir_index);
            }
        }
    }
    if picked.chem.chemical.temp.is_some() {
        actions.push(Action::Heat{target:combine_reservoir as u32 + 1, temp:picked.chem.chemical.temp.unwrap()});
    }

    state.replace(combine_reservoir, &picked.chem);
    return combine_reservoir as u32 + 1;
}