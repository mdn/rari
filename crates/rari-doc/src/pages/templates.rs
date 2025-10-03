use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Default, Deserialize)]
pub enum SpaBuildTemplate {
    #[default]
    SpaUnknown,
    SpaNotFound,
    SpaHomepage,
    SpaObservatoryLanding,
    SpaObservatoryAnalyze,
    SpaAdvertise,
    SpaPlusLanding,
    SpaPlusCollections,
    SpaPlusCollectionsFrequentlyViewed,
    SpaPlusUpdates,
    SpaPlusSettings,
    SpaPlusAiHelp,
    SpaPlay,
    SpaSearch,
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub enum DocPageRenderer {
    #[default]
    Doc,
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub enum BlogRenderer {
    #[default]
    BlogPost,
    BlogIndex,
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub enum ContributorSpotlightRenderer {
    #[default]
    ContributorSpotlight,
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub enum GenericRenderer {
    #[default]
    GenericDoc,
    GenericAbout,
    GenericCommunity,
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub enum SpaRenderer {
    #[default]
    SpaUnknown,
    SpaNotFound,
    SpaObservatoryLanding,
    SpaObservatoryAnalyze,
    SpaAdvertise,
    SpaPlusLanding,
    SpaPlusCollections,
    SpaPlusCollectionsFrequentlyViewed,
    SpaPlusUpdates,
    SpaPlusSettings,
    SpaPlusAiHelp,
    SpaPlay,
    SpaSearch,
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub enum HomeRenderer {
    #[default]
    Homepage,
}

#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub enum CurriculumRenderer {
    #[default]
    CurriculumDefault,
    CurriculumModule,
    CurriculumOverview,
    CurriculumLanding,
    CurriculumAbout,
}
