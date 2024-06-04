use neat::{CrossoverReproduction,  RandomlyMutable};

pub static mut LOADED_NNT: Option<neat::NeuralNetworkTopology<{ crate::AGENT_IN }, { crate::AGENT_OUT }>> = None;


#[derive(PartialEq, Clone, Debug, CrossoverReproduction, RandomlyMutable)]
pub struct DNA {
    pub network: neat::NeuralNetworkTopology<{ crate::AGENT_IN }, { crate::AGENT_OUT }>,
}

impl neat::Prunable for DNA {}

impl neat::GenerateRandom for DNA {
    fn gen_random(rng: &mut impl rand::Rng) -> Self {
        Self {
            network: neat::NeuralNetworkTopology::new(
                crate::MUTATION_RATE,
                crate::MUTATION_PASSES,
                rng,
            ),
            // network: unsafe { LOADED_NNT.clone().unwrap() },
        }
    }
}

pub struct PerformanceStats {
    pub high: f32,
    pub median: f32,
    pub low: f32,
}

pub struct PlottingNG<F: neat::NextgenFn<DNA>> {
    pub performance_stats: std::sync::Arc<std::sync::Mutex<Vec<PerformanceStats>>>,
    pub actual_ng: F,
}

impl<F: neat::NextgenFn<DNA>> neat::NextgenFn<DNA> for PlottingNG<F> {
    fn next_gen(&self, mut fitness: Vec<(DNA, f32)>) -> Vec<DNA> {
        // it's a bit slower because of sorting twice but I don't want to rewrite the nextgen.
        fitness.sort_by(|(_, fa), (_, fb)| fa.partial_cmp(fb).unwrap());

        let l = fitness.len();

        let high = fitness[l - 1].1;

        let median = fitness[l / 2].1;

        let low = fitness[0].1;

        let mut ps = self.performance_stats.lock().unwrap();
        ps.push(PerformanceStats { high, median, low });

        self.actual_ng.next_gen(fitness)
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
