use crate::engine::convention::convention_set::ConventionSet;
use crate::engine::convention::convention_tech::ConventionTech;

/// The H-Group convention framework.
pub struct HGroupConventionSet {
    techs: Vec<Box<dyn ConventionTech>>,
}

impl HGroupConventionSet {
    pub fn new(mut techs: Vec<Box<dyn ConventionTech>>) -> Self {
        techs.sort_by_key(|t| t.priority());
        HGroupConventionSet { techs }
    }
}

impl ConventionSet for HGroupConventionSet {
    fn techs(&self) -> &[Box<dyn ConventionTech>] {
        &self.techs
    }
}
