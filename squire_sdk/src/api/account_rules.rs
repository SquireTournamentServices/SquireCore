use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// The rules that dictate which tournaments should be recommended to a user
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rules {
    /// The user's location and distance they'd like to find tournaments from
    pub location: Option<LocationRule>,
    /// Keywords for tournament titles/types
    pub keywords: Option<Vec<String>>, // TODO: Likely change this later, make more flexible for types of matchings
    /// Companies that the user wants to play tournaments under
    pub companies: Option<Vec<Uuid>>,
}

/// Dictates the location a user will be recommended tournaments from
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationRule {
    /// The user's location
    pub central_location: Option<(f64, f64)>,
    /// The distance the user would like to find tournaments from
    pub distance_max: Option<f32>,
}

impl PartialEq for Rules {
    fn eq(&self, other: &Self) -> bool {
        self.keywords == other.keywords
            && self.companies == other.companies
            && self.location == other.location
    }
}

impl PartialEq for LocationRule {
    fn eq(&self, other: &Self) -> bool {
        (match (self.central_location, other.central_location) {
            (Some((lat1, lon1)), Some((lat2, lon2))) => 
                (lat1 - lat2).abs() < f64::EPSILON && (lon1 - lon2).abs() < f64::EPSILON,
            (None, None) => true,
            _ => false,
        }) &&
        (match (self.distance_max, other.distance_max) {
            (Some(location1), Some(location2)) => (location1 - location2).abs() < f32::EPSILON,
            (None, None) => true,
            _ => false,
        })
    }
}

impl Eq for Rules {} // TODO: Hacky solution because the struct uses floats. Likely revisit this
impl Eq for LocationRule {} // TODO: Same todo as above

impl Rules {
    /// Makes a new set of rules for a user
    pub fn new() -> Self {
        Rules {
            location: None,
            keywords: None,
            companies: None,
        }
    }

    /// Sets the user's location
    pub fn set_location(&mut self, location: LocationRule) {
        self.location = Some(location);
    }
    /// Deletes the user's location
    pub fn delete_location(&mut self) {
        self.location = None;
        
    }
    /// Gets the user's location
    pub fn get_location(&self) -> Option<LocationRule> {
        self.location.clone()
    }
    /// Adds a new game keyword
    pub fn add_game_keyword(&mut self, new_keyword: String) {
        match self.keywords {
            Some(ref mut keywords) => {
                keywords.push(new_keyword);
            }
            _ => {
                self.keywords = Some(vec![new_keyword]);
            }
        }
    } 
    /// Adds batch of game keywords
    pub fn add_batch_keywords(&mut self, new_keywords: Vec<String>) {
        match self.keywords {
            Some(ref mut keywords) => {
                for keyword in new_keywords {
                    keywords.push(keyword);
                }
            }
            _ => {
                self.keywords = Some(new_keywords);
            }
        }
    }
    /// Deletes a game keyword
    pub fn delete_game_keyword(&mut self, to_remove: String) {
        if let Some(pos) = self.keywords
            .as_ref()
            .and_then(|keywords| keywords.iter().position(|k| *k == to_remove)) {
                if let Some(keywords) = self.keywords.as_mut() {
                    let _  = keywords.remove(pos);
                    if keywords.is_empty() {
                        self.keywords = None;
                    }
                }
            }
    }
    /// Deletes batch of game kewywrods
    pub fn delete_batch_keywords(&mut self, to_remove: Vec<String>) {
        for remove in to_remove {
            self.delete_game_keyword(remove);
        }
    }
    /// Gets the game keywords
    pub fn get_keywords(&mut self) -> Option<Vec<String>> {
        if let Some(keywords) = self.keywords.as_mut() {
            keywords.sort();
        }
        self.keywords.clone()
    }

    /// Adds a company
    pub fn add_company(&mut self, new_company: Uuid) {
        match self.companies {
            Some(ref mut companies) => {
                companies.push(new_company);
            }
            None => {
                self.companies = Some(vec![new_company]);
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
}

impl LocationRule {
    /// Creates a new location rule struct
    pub fn new() -> Self {
        LocationRule { 
            central_location: None, 
            distance_max: None 
        }
    }

    /// Sets the user's central location
    pub fn set_central_location(&mut self, new_location: (f64, f64)) {
        self.central_location = Some(new_location);
    }
    
    /// Deletes the user's central location
    pub fn delete_central_location(&mut self) {
        self.central_location = None;
    }

    /// Gets the user's central location
    pub fn get_central_location(&self) -> Option<(f64, f64)> {
        self.central_location.clone()
    }

    /// Sets the user's maximum distance
    pub fn set_distance_max(&mut self, new_distance_max: f32) {
        self.distance_max = Some(new_distance_max);
    }

    /// Deletes the user's maximum distance
    pub fn delete_distance_max(&mut self) {
        self.distance_max = None;
    }

    /// Gets the user's maximum distance
    pub fn get_distance_max(&self) -> Option<f32> {
        self.distance_max.clone()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_new_rules() {
        let rules = Rules::new();
        assert!(rules.location.is_none());
        assert!(rules.keywords.is_none());
        assert!(rules.companies.is_none());
    }

    #[test]
    fn test_set_and_get_location() {
        let mut rules = Rules::new();
        rules.set_location(LocationRule::new());
        rules.location.as_mut().unwrap().set_central_location((2.0, 4.0));
        rules.location.as_mut().unwrap().set_distance_max(2.0);
        assert_eq!(rules.location.as_mut().unwrap().get_central_location(), Some((2.0, 4.0)));
        assert_eq!(rules.location.unwrap().get_distance_max(), Some(2.0));

    }

    #[test]
    fn test_delete_location() {
        let mut rules = Rules::new();
        rules.set_location(LocationRule::new());
        rules.location.as_mut().unwrap().set_central_location((2.0, 4.0));
        rules.location.as_mut().unwrap().set_distance_max(2.0);
        rules.delete_location();
        assert_eq!(rules.location, None);

    }

    #[test]
    fn test_add_get_and_delete_game_keyword() {
        let mut rules = Rules::new();
        let keyword = "abc".to_string();
        rules.add_game_keyword(keyword.clone());
        assert!(rules.get_keywords().unwrap().contains(&keyword));
        rules.delete_game_keyword(keyword.clone());
        assert!(!rules.get_keywords().unwrap_or_default().contains(&keyword));
    }

    #[test]
    fn test_add_get_and_delete_batch_keywords() {
        let mut rules = Rules::new();
        let keywords = vec!["abc".to_string(), "def".to_string()];
        rules.add_batch_keywords(keywords.clone());
        let mut i = 0;
        for keyword in &keywords {
            assert!(rules.get_keywords().unwrap().contains(keyword));
            if i == 0 {
                rules.delete_game_keyword(keyword.clone());
                i += 1;
            }
        }
        rules.delete_batch_keywords(keywords);
        assert!(rules.get_keywords().is_none() || rules.get_keywords().unwrap().is_empty());
    }

    #[test]
    fn test_add_get_and_delete_company() {
        let mut rules = Rules::new();
        let company_id = Uuid::new_v4();
        rules.add_company(company_id);
        assert!(rules.get_companies().unwrap().contains(&company_id));
        rules.delete_company(company_id);
        assert!(!rules.get_companies().unwrap_or_default().contains(&company_id));
    }

    #[test]
    fn test_add_get_and_delete_batch_companies() {
        let mut rules = Rules::new();
        let company_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        rules.add_batch_company(company_ids.clone());
        let mut i = 0;
        for id in &company_ids {
            assert!(rules.get_companies().unwrap().contains(id));
            if i == 0 {
                rules.delete_company(*id);
                i += 1;
            }
        }
        rules.delete_batch_companies(company_ids);
        assert!(rules.get_companies().is_none() || rules.get_companies().unwrap().is_empty());
    }

}
