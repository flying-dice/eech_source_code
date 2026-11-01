//! The host's rotor acoustics kernel (`eech_world::env::rotor_sound`, EEDCS-51, D3) against the MEASURED words of the ORIGINAL `update_local_helicopter_rotor_sounds`. SOURCE PREPARED: UNRUN BY ITS AUTHOR.
//!
//! The oracle is `tests/data/rotor_sound_oracle.tsv`: the inputs and the written output words, and the ordered pause and resume requests, that the original rotor probe recorded in the independently closed native
//! e54423b5c / driver 06de242ae runs -- the 12 AI and 15 flight model controlled sweep rows (including the five rows whose unassigned locals are zero by the canonical build's `-ftrivial-auto-var-init=zero`, F1) and
//! every distinct input of the natural 120 s record (208800 calls, 261 distinct AI inputs). The file is data copied from those retained records by a one-off extraction (header: provenance); NO second solver is written
//! here, and the kernel is not compared with itself. The natural record holds AI helicopters without a flight model only; every other shape is covered by the sweeps alone, which are world-input variation.
//!
//! What the kernel adds to the original is only its transport (32-bit words, the canonical arithmetic environment, the caller's floating-point state restored); the numbers must be the original's, bit for bit.

use eech_world::env::{rotor_sound, RotorPlayer, RotorSoundInput};

const ORACLE: &str = include_str!("data/rotor_sound_oracle.tsv");

fn word(text: &str) -> u32 {
    u32::from_str_radix(text, 16).unwrap_or_else(|e| panic!("{text:?} is not a word: {e}"))
}

fn value(text: &str) -> f32 {
    f32::from_bits(word(text))
}

#[test]
fn the_kernel_gives_the_measured_words_of_the_original_for_every_retained_input() {
    let mut checked = (0usize, 0usize, 0usize);
    for line in ORACLE.lines().filter(|line| !line.starts_with('#')) {
        let row = line.split('\t').collect::<Vec<_>>();
        let (source, index) = (row[0], row[1]);
        let what = format!("{source} {index}: {line}");
        let player = match row[2] {
            "0" => RotorPlayer::Ai,
            "1" => RotorPlayer::Local,
            "2" => RotorPlayer::Remote,
            other => panic!("{what}: player {other}"),
        };
        let have_flight_model = row[3] == "1";
        let inputs = row[4].split(',').map(value).collect::<Vec<f32>>();
        let sound = rotor_sound(&RotorSoundInput {
            player,
            rpm: inputs[0],
            acceleration: have_flight_model.then(|| [inputs[1], inputs[2], inputs[3]]),
            engines: [inputs[4], inputs[5], inputs[6]],
        })
        .unwrap_or_else(|e| panic!("{what}: the kernel refused: {e}"));

        // the written levels and pitches: a channel the helicopter had no child for ('-') wrote nothing in the original and is not compared
        for (channel, expected) in row[5].split(',').enumerate() {
            if expected != "-" {
                assert_eq!(
                    sound.amplification[channel].to_bits(),
                    word(expected),
                    "{what}: amplification of channel {channel}"
                );
            }
        }
        for (channel, expected) in row[6].split(',').enumerate() {
            if expected != "-" {
                assert_eq!(sound.pitch[channel].to_bits(), word(expected), "{what}: pitch of channel {channel}");
            }
        }
        assert_eq!(sound.write, [3, 3, 3, 3, 3, 1], "{what}: the write masks");
        assert_eq!(sound.pitch[5].to_bits(), 0, "{what}: the winding channel carries no pitch");

        // the ordered requests, with the delay words
        let actions = if row.get(7).copied().unwrap_or("").is_empty() {
            Vec::new()
        } else {
            row[7]
                .split(',')
                .map(|item| {
                    let part = item.split(':').collect::<Vec<_>>();
                    (part[0] == "1", part[1].parse::<usize>().expect("a channel"), word(part[2]))
                })
                .collect::<Vec<_>>()
        };
        let got = sound
            .actions
            .iter()
            .map(|action| (action.resume, action.channel, action.delay.to_bits()))
            .collect::<Vec<_>>();
        assert_eq!(got, actions, "{what}: the ordered requests");

        match source {
            "natural" => checked.0 += 1,
            "ai-sweep" => checked.1 += 1,
            _ => checked.2 += 1,
        }
    }
    // the retained record, not a sample: 261 distinct natural inputs, 12 AI and 15 flight model sweep rows
    assert_eq!(checked, (261, 12, 15));
}

#[test]
fn a_sweep_row_the_oracle_marks_as_the_canonical_builds_zero_has_both_unassigned_levels_zero_in_the_kernel() {
    // F1: rows 10 to 14 of the flight model sweep (the two engines within 1.0 of each other, or no APU): the original leaves right_turbine_amp and / or apu_amp unassigned and the canonical build's zero is the measured value
    let marked = ORACLE.lines().filter(|line| line.contains("# compiler_zero_init")).count();
    assert_eq!(marked, 5);
    for line in ORACLE.lines().filter(|line| line.contains("# compiler_zero_init")) {
        let row = line.split('\t').collect::<Vec<_>>();
        let inputs = row[4].split(',').map(value).collect::<Vec<f32>>();
        let sound = rotor_sound(&RotorSoundInput {
            player: RotorPlayer::Local,
            rpm: inputs[0],
            acceleration: Some([inputs[1], inputs[2], inputs[3]]),
            engines: [inputs[4], inputs[5], inputs[6]],
        })
        .unwrap();
        let single_sound = (inputs[4] - inputs[5]).abs() <= 1.0;
        let no_apu = inputs[6] <= 0.0;
        assert!(single_sound || no_apu, "{line}");
        if single_sound {
            assert_eq!(sound.amplification[3].to_bits(), 0, "{line}: right_turbine_amp");
        }
        if no_apu {
            assert_eq!(sound.amplification[4].to_bits(), 0, "{line}: apu_amp");
        }
    }
}
