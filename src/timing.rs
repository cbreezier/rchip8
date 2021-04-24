use std::time::Instant;
use std::vec::Vec;

pub struct TimedSystem {
    name: &'static str,
    cycle_duration_nanos: u64,
    elapsed_cycles: u64,
}

impl TimedSystem {
    pub fn new(name: &'static str, cycle_speed_hz: u64) -> Self {
        Self {
            name,
            cycle_duration_nanos: 1_000_000_000 / cycle_speed_hz,
            elapsed_cycles: 0,
        }
    }

    fn next_cycle_nanos(&self) -> u64 {
        return self.cycle_duration_nanos * (self.elapsed_cycles + 1);
    }

    // Number of cycles that can be executed until we are > the target_nanos
    fn num_cycles_until(&self, target_nanos: u64) -> u64 {
        let next_nanos = self.next_cycle_nanos();
        if target_nanos == next_nanos {
            return 1;
        }

        return TimedSystem::divide_round_up(
            target_nanos - next_nanos,
            self.cycle_duration_nanos,
        );
    }

    fn divide_round_up(a: u64, b: u64) -> u64 {
        return (a + b - 1) / b;
    }
}

pub struct Instruction {
    pub name: &'static str,
    pub cycles: u64,
}

pub struct Timing {
    start_time: Instant,
    systems: Vec<TimedSystem>,
}

impl Timing {

    pub fn new(
        current_time: Instant,
        systems: Vec<TimedSystem>,
    ) -> Self {
        Self {
            start_time: current_time,
            systems,
        }
    }

    pub fn get_instructions(&mut self, current_time: Instant) -> Vec<Instruction> {
        let required_nanos = (current_time - self.start_time).as_nanos();

        let mut results: Vec<Instruction> = Vec::new();
        let mut watchdog = 0;
        loop {
            if watchdog > 1_000 {
                panic!("Infinite loop");
            }
            watchdog += 1;

            // Sort systems by the soonest next cycle
            self.systems
                .sort_by(|a, b| a.next_cycle_nanos().cmp(&b.next_cycle_nanos()));

            for system in &self.systems {
                println!("Sorted {} at cycle {}", system.name, system.next_cycle_nanos());
            }

            let soonest_nanos = self.systems[0].next_cycle_nanos();
            let next_soonest_nanos = self.systems[1].next_cycle_nanos();

            if u128::from(soonest_nanos) >= required_nanos {
                break;
            }

            // TODO currently assume there is more than one system

            // We put as many cycles as we can from the soonest system, until
            // it is no longer the soonest system
            let num_cycles = self.systems[0].num_cycles_until(next_soonest_nanos);

            // Add it to our results
            println!("Adding instruction {} for {} cycles", self.systems[0].name, num_cycles);
            results.push(Instruction {
                name: self.systems[0].name.clone(),
                cycles: num_cycles,
            });

            // Update our systems to take into account the cycles that will be executed
            self.systems[0].elapsed_cycles += num_cycles;
        }

        println!("--- Emitted {} instructions", results.len());
        return results;
    }
}