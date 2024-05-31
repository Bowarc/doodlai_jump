use neat::{CrossoverReproduction, DivisionReproduction, RandomlyMutable};

pub const IN: usize = 12;
pub const OUT: usize = 3; // None, Left, right

#[derive(PartialEq, Clone, Debug, DivisionReproduction, RandomlyMutable, CrossoverReproduction)]
pub struct DNA {
    pub network: neat::NeuralNetworkTopology<IN, OUT>,
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
    pub network: neat::NeuralNetwork<IN, OUT>,
}

impl From<&DNA> for Agent {
    fn from(dna: &DNA) -> Self {
        Self {
            network: (&dna.network).into(),
        }
    }
}
