use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::json::{
    JsonBlogPostPage, JsonContributorSpotlightPage, JsonCurriculumPage, JsonDocPage,
    JsonGenericPage, JsonHomePage, JsonSpaPage,
};
use super::types::{curriculum, generic};

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

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "renderer")]
pub enum SpaPage {
    SpaUnknown(JsonSpaPage),
    SpaNotFound(JsonSpaPage),
    SpaHomepage(JsonSpaPage),
    SpaObservatoryLanding(JsonSpaPage),
    SpaObservatoryAnalyze(JsonSpaPage),
    SpaAdvertise(JsonSpaPage),
    SpaPlusLanding(JsonSpaPage),
    SpaPlusCollections(JsonSpaPage),
    SpaPlusCollectionsFrequentlyViewed(JsonSpaPage),
    SpaPlusUpdates(JsonSpaPage),
    SpaPlusSettings(JsonSpaPage),
    SpaPlusAiHelp(JsonSpaPage),
    SpaPlay(JsonSpaPage),
    SpaSearch(JsonSpaPage),
}

impl SpaPage {
    pub fn from_page_and_template(page: JsonSpaPage, template: SpaBuildTemplate) -> Self {
        match template {
            SpaBuildTemplate::SpaUnknown => Self::SpaUnknown(page),
            SpaBuildTemplate::SpaNotFound => Self::SpaNotFound(page),
            SpaBuildTemplate::SpaHomepage => Self::SpaHomepage(page),
            SpaBuildTemplate::SpaObservatoryLanding => Self::SpaObservatoryLanding(page),
            SpaBuildTemplate::SpaObservatoryAnalyze => Self::SpaObservatoryAnalyze(page),
            SpaBuildTemplate::SpaAdvertise => Self::SpaAdvertise(page),
            SpaBuildTemplate::SpaPlusLanding => Self::SpaPlusLanding(page),
            SpaBuildTemplate::SpaPlusCollections => Self::SpaPlusCollections(page),
            SpaBuildTemplate::SpaPlusCollectionsFrequentlyViewed => {
                Self::SpaPlusCollectionsFrequentlyViewed(page)
            }
            SpaBuildTemplate::SpaPlusUpdates => Self::SpaPlusUpdates(page),
            SpaBuildTemplate::SpaPlusSettings => Self::SpaPlusSettings(page),
            SpaBuildTemplate::SpaPlusAiHelp => Self::SpaPlusAiHelp(page),
            SpaBuildTemplate::SpaPlay => Self::SpaPlay(page),
            SpaBuildTemplate::SpaSearch => Self::SpaSearch(page),
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "renderer")]
pub enum GenericPage {
    GenericDoc(JsonGenericPage),
    GenericAbout(JsonGenericPage),
    GenericCommunity(JsonGenericPage),
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "renderer")]
pub enum DocPage {
    Doc(JsonDocPage),
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "renderer")]
pub enum BlogPage {
    BlogPost(JsonBlogPostPage),
    BlogIndex(JsonBlogPostPage),
}

impl GenericPage {
    pub fn from_page_and_template(page: JsonGenericPage, template: generic::Template) -> Self {
        match template {
            generic::Template::GenericDoc => Self::GenericDoc(page),
            generic::Template::GenericAbout => Self::GenericAbout(page),
            generic::Template::GenericCommunity => Self::GenericCommunity(page),
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "renderer")]
pub enum ContributorSpotlightPage {
    ContributorSpotlight(JsonContributorSpotlightPage),
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "renderer")]
pub enum HomePage {
    Homepage(JsonHomePage),
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "renderer")]
pub enum CurriculumPage {
    CurriculumDefault(JsonCurriculumPage),
    CurriculumModule(JsonCurriculumPage),
    CurriculumOverview(JsonCurriculumPage),
    CurriculumLanding(JsonCurriculumPage),
    CurriculumAbout(JsonCurriculumPage),
}

impl CurriculumPage {
    pub fn from_page_and_template(
        page: JsonCurriculumPage,
        template: curriculum::Template,
    ) -> Self {
        match template {
            curriculum::Template::Module => Self::CurriculumModule(page),
            curriculum::Template::Overview => Self::CurriculumOverview(page),
            curriculum::Template::Landing => Self::CurriculumLanding(page),
            curriculum::Template::About => Self::CurriculumAbout(page),
            curriculum::Template::Default => Self::CurriculumDefault(page),
        }
    }
}
