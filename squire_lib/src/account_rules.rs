use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// The rules that dictate which tournaments should be recommended to a user
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rules {
    /// The user's location
    pub location: Option<(f64, f64)>,
    /// The recommended distance from the user's location
    pub distance_max: Option<f32>,
    /// Keywords for tournament titles/types
    pub game_keywords: Option<Vec<String>>, // Likely change this later, make more flexible for types of matchings
    /// Companies that the user wants to play tournaments under
    pub companies: Option<Vec<Uuid>>,
    /// The number of rules set by the user
    num_rules_set: u128,
}

impl PartialEq for Rules {
    fn eq(&self, other: &Self) -> bool {
        self.game_keywords == other.game_keywords
            && self.companies == other.companies
            && self.num_rules_set == other.num_rules_set
            && match (self.location, other.location) {
                (Some((lat1, lon1)), Some((lat2, lon2))) => (lat1 - lat2).abs() < f64::EPSILON && (lon1 - lon2).abs() < f64::EPSILON,
                (None, None) => true,
                _ => false,
            }
            && match (self.distance_max, other.distance_max) {
                (Some(dm1), Some(dm2)) => (dm1 - dm2).abs() < f32::EPSILON,
                (None, None) => true,
                _ => false,
            }
    }
}

impl Eq for Rules {} // Hacky solution because the struct uses floats. Likely revisit this

impl Rules {
    /// Makes a new set of rules for a user
    pub fn new() -> Self {
        Rules {
            location: None,
            distance_max: None,
            game_keywords: None,
            companies: None,
            num_rules_set: 0,
        }
    }

    /// Sets the user's location
    pub fn set_location(&mut self, location: (f64, f64)) {
        if self.location.is_none() {
            self.increment_num_rules();
        }
        self.location = Some(location);
    }
    /// Deletes the user's location
    pub fn delete_location(&mut self) {
        if self.location.is_some() {
            self.location = None;
            self.decrement_num_rules();
        }
    }
    /// Gets the user's location
    pub fn get_location(&self) -> Option<(f64, f64)> {
        self.location.clone()
    }
    
    /// Sets the maximum distance of recommended tournaments
    pub fn set_distance_max(&mut self, distance_max: f32) {
        if self.distance_max.is_none() {
            self.increment_num_rules();
        }
        self.distance_max = Some(distance_max);
    }
    /// Deletes the maximum distance of recommended tournaments
    pub fn delete_distance_max(&mut self) {
        if self.distance_max.is_some() {
            self.distance_max = None;
            self.decrement_num_rules();
        }
    }
    /// Gets the maximum distance of recommended tournaments
    pub fn get_distance_max(&self) -> Option<f32> {
        self.distance_max.clone()
    }

    /// Adds a new game keyword
    pub fn add_game_keyword(&mut self, new_keyword: String) {
        match self.game_keywords {
            Some(ref mut keywords) => {
                keywords.push(new_keyword);
            }
            _ => {
                self.game_keywords = Some(vec![new_keyword]);
                self.increment_num_rules();
            }
        }
    } 
    /// Adds batch of game keywords
    pub fn add_batch_game_keywords(&mut self, new_keywords: Vec<String>) {
        match self.game_keywords {
            Some(ref mut keywords) => {
                for keyword in new_keywords {
                    keywords.push(keyword);
                }
            }
            _ => {
                self.game_keywords = Some(new_keywords);
                self.increment_num_rules();
            }
        }
    }
    /// Deletes a game keyword
    pub fn delete_game_keyword(&mut self, to_remove: String) {
        if let Some(pos) = self.game_keywords
            .as_ref()
            .and_then(|keywords| keywords.iter().position(|k| *k == to_remove)) {
                if let Some(keywords) = self.game_keywords.as_mut() {
                    let _  = keywords.remove(pos);
                    if keywords.is_empty() {
                        self.game_keywords = None;
                        self.decrement_num_rules();
                    }
                }
            }
    }
    /// Deletes batch of game kewywrods
    pub fn delete_batch_game_keywords(&mut self, to_remove: Vec<String>) {
        for remove in to_remove {
            self.delete_game_keyword(remove);
        }
    }
    /// Gets the game keywords
    pub fn get_game_keywords(&mut self) -> Option<Vec<String>> {
        if let Some(keywords) = self.game_keywords.as_mut() {
            keywords.sort();
        }
        self.game_keywords.clone()
    }

    /// Adds a company
    pub fn add_company(&mut self, new_company: Uuid) {
        match self.companies {
            Some(ref mut companies) => {
                companies.push(new_company);
            }
            None => {
                self.companies = Some(vec![new_company]);
                self.increment_num_rules();
            }
        }
    }
    /// Adds batch of companies
    pub fn add_batch_company(&mut self, new_companies: Vec<Uuid>) {
        match self.companies {
            Some(ref mut companies) => {
                for new_company in new_companies {
                    companies.push(new_company);
                
                }
            }
            _ => {
                self.companies = Some(new_companies);
                self.increment_num_rules();
            }
        }
    }
    /// Deletes a company
    pub fn delete_company(&mut self, to_remove: Uuid) {
        if let Some(pos) = self.companies 
            .as_ref()
            .and_then(|companies| companies.iter().position(|k| *k == to_remove)) {
                if let Some(companies) = self.companies.as_mut() {
                    let _ = companies.remove(pos);
                    if companies.is_empty() {
                        self.companies = None;
                        self.decrement_num_rules();
                    }
                }
                
            }
    }
    /// Deletes batch of companies
    pub fn delete_batch_companies(&mut self, to_remove: Vec<Uuid>) {
        for remove in to_remove {
            self.delete_company(remove);
        }
    }
    /// Gets the companies
    pub fn get_companies(&self) -> Option<Vec<Uuid>> {
        self.companies.clone()
    }

    /// Gets the number of rules set for the user
    pub fn get_num_rules_set(&self) -> u128 {
        self.num_rules_set
    }
    /// Increments the number of rules
    fn increment_num_rules(&mut self) {
        self.num_rules_set += 1;
    }
    /// Decrements the number of rules
    fn decrement_num_rules(&mut self) {
        if self.num_rules_set > 0 {
            self.num_rules_set -= 1;
        } 
    }
    /// Validates and potentially corrects the number of rules for a user
    pub fn validate_num_rules(&mut self) {
        let mut num_rules = 0;
        if !(self.location.is_none()){
            num_rules += 1;
        }
        if !(self.distance_max.is_none()) {
            num_rules += 1;
        }
        if !(self.game_keywords.is_none()) {
            num_rules += 1;
        }
        if !(self.companies.is_none()) {
            num_rules += 1;
        }
        if self.num_rules_set != num_rules {
            self.num_rules_set = num_rules;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_new_rules() {
        let rules = Rules::new();
        assert_eq!(rules.num_rules_set, 0);
        assert!(rules.location.is_none());
        assert!(rules.distance_max.is_none());
        assert!(rules.game_keywords.is_none());
        assert!(rules.companies.is_none());
    }

    #[test]
    fn test_set_and_get_location() {
        let mut rules = Rules::new();
        let location = (45.0, -75.0);
        rules.set_location(location);
        assert_eq!(rules.get_location(), Some(location));
        assert_eq!(rules.get_num_rules_set(), 1);
    }

    #[test]
    fn test_delete_location() {
        let mut rules = Rules::new();
        rules.set_location((45.0, -75.0));
        rules.delete_location();
        assert!(rules.location.is_none());
        assert_eq!(rules.get_num_rules_set(), 0);
    }

    #[test]
    fn test_set_and_get_distance_max() {
        let mut rules = Rules::new();
        let distance_max = 10.0;
        rules.set_distance_max(distance_max);
        assert_eq!(rules.get_distance_max(), Some(distance_max));
        assert_eq!(rules.get_num_rules_set(), 1);
    }

    #[test]
    fn test_delete_distance_max() {
        let mut rules = Rules::new();
        rules.set_distance_max(10.0);
        rules.delete_distance_max();
        assert!(rules.distance_max.is_none());
        assert_eq!(rules.get_num_rules_set(), 0);
    }

    #[test]
    fn test_add_get_and_delete_game_keyword() {
        let mut rules = Rules::new();
        let keyword = "abc".to_string();
        rules.add_game_keyword(keyword.clone());
        assert!(rules.get_game_keywords().unwrap().contains(&keyword));
        assert_eq!(rules.get_num_rules_set(), 1);
        rules.delete_game_keyword(keyword.clone());
        assert!(!rules.get_game_keywords().unwrap_or_default().contains(&keyword));
        assert_eq!(rules.get_num_rules_set(), 0);
    }

    #[test]
    fn test_add_get_and_delete_batch_game_keywords() {
        let mut rules = Rules::new();
        let keywords = vec!["abc".to_string(), "def".to_string()];
        rules.add_batch_game_keywords(keywords.clone());
        assert_eq!(rules.get_num_rules_set(), 1);
        let mut i = 0;
        for keyword in &keywords {
            assert!(rules.get_game_keywords().unwrap().contains(keyword));
            if i == 0 {
                rules.delete_game_keyword(keyword.clone());
                i += 1;
            }
        }
        assert_eq!(rules.get_num_rules_set(), 1);
        rules.delete_batch_game_keywords(keywords);
        assert!(rules.get_game_keywords().is_none() || rules.get_game_keywords().unwrap().is_empty());
        assert_eq!(rules.get_num_rules_set(), 0);
    }

    #[test]
    fn test_add_get_and_delete_company() {
        let mut rules = Rules::new();
        let company_id = Uuid::new_v4();
        rules.add_company(company_id);
        assert!(rules.get_companies().unwrap().contains(&company_id));
        assert_eq!(rules.get_num_rules_set(), 1);
        rules.delete_company(company_id);
        assert!(!rules.get_companies().unwrap_or_default().contains(&company_id));
        assert_eq!(rules.get_num_rules_set(), 0);
    }

    #[test]
    fn test_add_get_and_delete_batch_companies() {
        let mut rules = Rules::new();
        let company_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        rules.add_batch_company(company_ids.clone());
        assert_eq!(rules.get_num_rules_set(), 1);
        let mut i = 0;
        for id in &company_ids {
            assert!(rules.get_companies().unwrap().contains(id));
            if i == 0 {
                rules.delete_company(*id);
                i += 1;
            }
        }
        assert_eq!(rules.get_num_rules_set(), 1);
        rules.delete_batch_companies(company_ids);
        assert!(rules.get_companies().is_none() || rules.get_companies().unwrap().is_empty());
        assert_eq!(rules.get_num_rules_set(), 0);
    }

    #[test]
    fn test_num_rules_set_increments_and_decrements_correctly() {
        let mut rules = Rules::new();
        rules.set_location((45.0, -75.0));
        rules.set_distance_max(10.0);
        rules.add_game_keyword("abc".to_string());
        let company_id = Uuid::new_v4();
        rules.add_company(company_id);
        assert_eq!(rules.get_num_rules_set(), 4);

        rules.delete_location();
        rules.delete_distance_max();
        rules.delete_game_keyword("abc".to_string());
        rules.delete_company(company_id);
        assert_eq!(rules.get_num_rules_set(), 0);
    }

    #[test]
    fn test_validate_num_rules() {
        let mut rules = Rules::new();
        rules.set_location((45.0, -75.0));
        rules.set_distance_max(10.0);
        rules.add_game_keyword("abc".to_string());
        let company_id = Uuid::new_v4();
        rules.add_company(company_id);

        rules.num_rules_set = 999; 
        rules.validate_num_rules();
        assert_eq!(rules.get_num_rules_set(), 4); 
    }
    
    #[test]
    fn test_num_rules_set_does_not_go_below_zero() {
        let mut rules = Rules::new();
        rules.decrement_num_rules(); 
        assert_eq!(rules.num_rules_set, 0);
    }
}
