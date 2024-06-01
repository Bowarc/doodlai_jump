use neat::{CrossoverReproduction, DivisionReproduction, RandomlyMutable};

#[derive(PartialEq, Clone, Debug, DivisionReproduction, RandomlyMutable, CrossoverReproduction)]
pub struct DNA {
    pub network: neat::NeuralNetworkTopology<{ crate::AGENT_IN }, { crate::AGENT_OUT }>,
}

impl neat::Prunable for DNA {}

impl neat::GenerateRandom for DNA {
    fn gen_random(rng: &mut impl rand::Rng) -> Self {
        Self {
            network: neat::NeuralNetworkTopology::new(0.01, 3, rng),
            // network: neat::NeuralNetworkTopology::new(0.1, 3, rng),
            // network: unsafe { crate::LOADED_NNT.clone().unwrap() },
        }
    }
}

#[derive(Debug)]
pub struct Agent {
    pub network: neat::NeuralNetwork<{ crate::AGENT_IN }, { crate::AGENT_OUT }>,
}

impl From<&DNA> for Agent {
    fn from(dna: &DNA) -> Self {
        Self {
            network: (&dna.network).into(),
        }
    }
}
