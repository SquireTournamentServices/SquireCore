use uuid::Uuid;

/// The rules that dictate which tournaments should be recommended to a user
pub struct Rules {
    /// The user's location
    pub location: Option<(f64, f64)>,
    /// The recommended distance from the user's location
    pub distance_max: Option<f32>,
    /// Keywords for tournament titles/types
    pub game_keywords: Option<Vec<String>>, // likely change this later, make more flexible for types of matchings
    /// Companies that the user wants to play tournaments under
    pub companies: Option<Vec<Uuid>>,
    /// The number of rules set by the user
    num_rules_set: u128,
}
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

    /// Adds a new keyword
    pub fn add_game_keyword(&mut self, new_keyword: String) {
        match self.game_keywords {
            Some(ref mut keywords) => {
                keywords.push(new_keyword);
            }
            None => {
                self.game_keywords = Some(vec![new_keyword]);
                self.increment_num_rules();
            }
        }
    } 
    /// Deletes a specified keyword
    pub fn delete_game_keyword(&mut self, to_remove: String) {
        if let Some(pos) = self.game_keywords
            .as_ref()
            .and_then(|keywords| keywords.iter().position(|k| *k == to_remove)) {
                if let Some(keywords) = self.game_keywords.as_mut() {
                    keywords.remove(pos);
                    if keywords.is_empty() {
                        self.game_keywords = None;
                        self.decrement_num_rules();
                    }
                }
            }
    }
    /// Gets the keywords
    pub fn get_game_keywords(&mut self) -> Option<Vec<String>> {
        if let Some(keywords) = self.game_keywords.as_mut() {
            keywords.sort();
        }
        self.game_keywords
    }

    /// Adds a company
    pub fn add_company_id(&mut self, new_company: Uuid) {
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
    /// Deletes a company
    pub fn delete_company_id(&mut self, to_remove: Uuid) {
        if let Some(pos) = self.companies 
            .as_ref()
            .and_then(|companies| companies.iter().position(|k| *k == to_remove)) {
                if let Some(companies) = self.companies.as_mut() {
                    companies.remove(pos);
                    if companies.is_empty() {
                        self.companies = None;
                        self.decrement_num_rules();
                    }
                }
                
            }
    }
    /// Gets the company ids
    pub fn get_company_ids(&self) -> Option<Vec<Uuid>> {
        self.companies
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
        if !(self.num_rules_set != num_rules) {
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
    }

    #[test]
    fn test_set_location() {
        let mut rules = Rules::new();
        rules.set_location((45.0, -75.0));
        assert_eq!(rules.num_rules_set, 1);
        assert!(rules.location.is_some());
    }

    #[test]
    fn test_delete_location() {
        let mut rules = Rules::new();
        rules.set_location((45.0, -75.0));
        rules.delete_location();
        assert_eq!(rules.num_rules_set, 0);
        assert!(rules.location.is_none());
    }

    #[test]
    fn test_set_distance_max() {
        let mut rules = Rules::new();
        rules.set_distance_max(10.0);
        assert_eq!(rules.num_rules_set, 1);
        assert!(rules.distance_max.is_some());
    }

    #[test]
    fn test_add_and_delete_game_keyword() {
        let mut rules = Rules::new();
        let keyword = "chess".to_string();
        rules.add_game_keyword(keyword.clone());
        assert_eq!(rules.num_rules_set, 1);
        rules.delete_game_keyword(keyword);
        assert_eq!(rules.num_rules_set, 0);
        assert!(rules.game_keywords.is_none());
    }

    #[test]
    fn test_add_and_delete_company_id() {
        let mut rules = Rules::new();
        let company_id = Uuid::new_v4();
        rules.add_company_id(company_id);
        assert_eq!(rules.num_rules_set, 1);
        rules.delete_company_id(company_id);
        assert_eq!(rules.num_rules_set, 0);
        assert!(rules.companies.is_none());
    }

    #[test]
    fn test_num_rules_set_does_not_go_below_zero() {
        let mut rules = Rules::new();
        rules.decrement_num_rules(); 
        assert_eq!(rules.num_rules_set, 0);
    }
    
}
