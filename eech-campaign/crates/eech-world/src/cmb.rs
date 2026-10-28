//! The host's physical combat-resolution kernel (EEDCS-46, D2).  Its first question is the damage FORECAST: the damage a weapon is capable of against a target with no weapon
//! in flight, which the campaign's target selection asks (`wn_tgt.c`).  Kept in C (`csrc/cmb_step.c`, EECH's own arithmetic expression for expression) and driven here one
//! question at a time.  The question is pure and takes no sample of any random stream; the campaign sends everything it needs by value (the weapon's damage figures, the
//! target's armour level, whether the target is a player and the player's damage setting).  The model here is the reference one: a host with another model (a DCS host)
//! answers from its own state and needs neither this arithmetic nor these figures.

use std::ffi::{c_float, c_int};

/// `cf_forecast_in`.
#[repr(C)]
struct RawForecastIn {
    weapon_warhead: c_int,
    warhead: c_int,
    damage_capability: c_int,
    armor_penetration_capability: c_int,
    burst_duration: c_float,
    armour_level: c_int,
    not_ai: c_int,
    player_modifier: c_float,
    damage_modifier: c_float,
    velocity: c_float,
}

/// `cf_forecast_out`.
#[repr(C)]
#[derive(Default)]
struct RawForecastOut {
    capability: c_int,
    path: c_int,
    raw: c_int,
    warhead: c_int,
    damage_modifier: c_float,
    penetration: c_float,
    armour: c_float,
    random_modifier: c_float,
    failed: c_int,
}

extern "C" {
    fn eech_cmb_forecast(input: *const RawForecastIn, output: *mut RawForecastOut);
}

/// The forecast question, by value: the weapon's own warhead type, the figures the forecast is made from (the weapon's own, or for a conventional-munitions weapon those of
/// the munition it scatters, as the campaign resolved them), the target's armour level, whether the target is not an AI and, then, the player's damage setting, and the
/// two arguments of the original function (the damage modifier and the velocity).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForecastInput {
    pub weapon_warhead: i32,
    pub warhead: i32,
    pub damage_capability: i32,
    pub armor_penetration_capability: i32,
    pub burst_duration: f32,
    pub armour_level: i32,
    pub not_ai: bool,
    pub player_modifier: f32,
    pub damage_modifier: f32,
    pub velocity: f32,
}

/// The capability and the values the original's observer hooks saw (the fixture build asks for them): the arithmetic path (0 none, 1 the weapon pierces the armour, 2
/// high-explosive splash), the capability before the player's setting, the warhead the arithmetic used, the damage modifier as it stood at the end, the penetration capability,
/// the armour thickness and the random modifier (1: a forecast has none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForecastOutcome {
    pub capability: i32,
    pub path: i32,
    pub raw: i32,
    pub warhead: i32,
    pub damage_modifier: f32,
    pub penetration: f32,
    pub armour: f32,
    pub random_modifier: f32,
}

/// The damage a weapon is capable of against a target (no weapon in flight).  An error for a figure that is not finite, an armour level below zero (the original takes a
/// fractional power of it), and an intermediate value the original could not convert to an integer (its result there is undefined).
pub fn forecast(input: &ForecastInput) -> Result<ForecastOutcome, String> {
    if !(input.burst_duration.is_finite() && input.player_modifier.is_finite() && input.damage_modifier.is_finite() && input.velocity.is_finite()) {
        return Err("a forecast needs finite figures".into());
    }
    if input.armour_level < 0 {
        return Err(format!(
            "a forecast against armour level {} (below zero) has no defined result",
            input.armour_level
        ));
    }
    let raw = RawForecastIn {
        weapon_warhead: input.weapon_warhead,
        warhead: input.warhead,
        damage_capability: input.damage_capability,
        armor_penetration_capability: input.armor_penetration_capability,
        burst_duration: input.burst_duration,
        armour_level: input.armour_level,
        not_ai: c_int::from(input.not_ai),
        player_modifier: input.player_modifier,
        damage_modifier: input.damage_modifier,
        velocity: input.velocity,
    };
    let mut out = RawForecastOut::default();
    // SAFETY: both structures are valid for the call, which keeps nothing.
    unsafe { eech_cmb_forecast(&raw, &mut out) };
    if out.failed != 0 {
        return Err("the forecast arithmetic left a value that is not an integer the original could convert (non-finite or out of range)".into());
    }
    Ok(ForecastOutcome {
        capability: out.capability,
        path: out.path,
        raw: out.raw,
        warhead: out.warhead,
        damage_modifier: out.damage_modifier,
        penetration: out.penetration,
        armour: out.armour,
        random_modifier: out.random_modifier,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // The expected values below are worked out by hand from the original expressions of `weapon_damage_capability` (damage.c, the `wpn == NULL` path), for figures chosen so
    // that no value sits on an integer boundary; they are not read back from this code.
    const SOLID_SHOT: i32 = 3;
    const HE_ANTI_TANK: i32 = 4;
    const HE_DUAL_PURPOSE: i32 = 5;
    const HE_PLAIN: i32 = 6;
    const HE_ANTI_AIRCRAFT: i32 = 7;
    const CONVENTIONAL: i32 = 8;

    fn input(warhead: i32, damage: i32, penetration: i32, armour: i32, modifier: f32) -> ForecastInput {
        ForecastInput {
            weapon_warhead: warhead,
            warhead,
            damage_capability: damage,
            armor_penetration_capability: penetration,
            burst_duration: 0.0,
            armour_level: armour,
            not_ai: false,
            player_modifier: 1.0,
            damage_modifier: modifier,
            velocity: 1.0,
        }
    }

    #[test]
    fn a_weapon_that_pierces_the_armour_does_its_full_damage() {
        // penetration 20 >= armour 10 and the callers' damage modifier 0.0: the damage figure, path 1
        let outcome = forecast(&input(SOLID_SHOT, 12, 20, 10, 0.0)).unwrap();
        assert_eq!((outcome.capability, outcome.path, outcome.raw, outcome.warhead), (12, 1, 12, SOLID_SHOT));
        assert_eq!(
            (outcome.penetration, outcome.armour, outcome.random_modifier, outcome.damage_modifier),
            (20.0, 10.0, 1.0, 0.0)
        );
    }

    #[test]
    fn a_weapon_that_cannot_pierce_and_is_not_explosive_does_nothing() {
        // penetration 5 < armour 10, not a high-explosive warhead: no arithmetic path runs and the capability is zero
        let outcome = forecast(&input(SOLID_SHOT, 12, 5, 10, 0.0)).unwrap();
        assert_eq!((outcome.capability, outcome.path, outcome.raw), (0, 0, 0));
        // a damage modifier of 0.01 or more also stops the pierce, whatever the penetration
        let outcome = forecast(&input(SOLID_SHOT, 12, 500, 10, 0.5)).unwrap();
        assert_eq!((outcome.capability, outcome.path), (0, 0));
        // a weapon with no damage figure (a smoke grenade) pierces for zero
        let outcome = forecast(&input(1, 0, 0, 0, 0.0)).unwrap();
        assert_eq!((outcome.capability, outcome.path), (0, 1));
    }

    #[test]
    fn a_non_explosive_weapon_is_scaled_by_the_velocity_argument_and_an_explosive_one_is_not() {
        // velocity 2.0: penetration 10 * 1 * 2.0 = 20 >= armour 15, damage 12 * 1 * 2.0 = 24
        let mut solid = input(SOLID_SHOT, 12, 10, 15, 0.0);
        solid.velocity = 2.0;
        assert_eq!(forecast(&solid).unwrap().capability, 24);
        // a high-explosive warhead forces the velocity to 1
        let mut explosive = input(HE_PLAIN, 12, 10, 15, 0.0);
        explosive.velocity = 2.0;
        explosive.armor_penetration_capability = 20;
        let outcome = forecast(&explosive).unwrap();
        assert_eq!((outcome.capability, outcome.path, outcome.penetration), (12, 1, 20.0));
    }

    #[test]
    fn high_explosive_splash_follows_the_warhead_and_the_armour() {
        // damage modifier 0.5 stops the pierce.  The splash: 1 - 0.5^(((w - 4 + 1) / 3)^1.5), damage 100 against armour 4 (a thickness of 4^1.5 = 8, so no armour limit)
        for (warhead, expected_damage, expected_modifier) in [
            (HE_ANTI_TANK, 12, 0.124_881_73_f32),
            (HE_DUAL_PURPOSE, 31, 0.314_290_72),
            (HE_PLAIN, 50, 0.5),
            (HE_ANTI_AIRCRAFT, 65, 0.656_019_4),
        ] {
            let outcome = forecast(&input(warhead, 100, 1, 4, 0.5)).unwrap();
            assert_eq!(
                (outcome.capability, outcome.path, outcome.raw, outcome.warhead),
                (expected_damage, 2, expected_damage, warhead),
                "warhead {warhead}"
            );
            assert!(
                (outcome.damage_modifier - expected_modifier).abs() < 1e-6,
                "warhead {warhead}: {}",
                outcome.damage_modifier
            );
        }
        // against armour 100 (a thickness of 1000) the armour limits it: 100 * modifier * (100 * modifier / 1000)
        for (warhead, expected_damage) in [(HE_ANTI_TANK, 0), (HE_DUAL_PURPOSE, 0), (HE_PLAIN, 2), (HE_ANTI_AIRCRAFT, 4)] {
            assert_eq!(
                forecast(&input(warhead, 100, 1, 100, 0.5)).unwrap().capability,
                expected_damage,
                "warhead {warhead}"
            );
        }
        // armour 0 divides by zero: the limit is infinite and does not apply; but 0 / 0 is not a number and is refused rather than converted
        assert_eq!(forecast(&input(HE_DUAL_PURPOSE, 100, 1, 0, 0.5)).unwrap().capability, 31);
        assert!(forecast(&input(HE_DUAL_PURPOSE, 0, 1, 0, 0.5)).unwrap_err().contains("not an integer"));
    }

    #[test]
    fn a_conventional_munitions_weapon_takes_the_munitions_figures_scaled_by_the_burst() {
        // the weapon's own warhead is conventional munitions; the campaign sends the munition's figures: damage 30 * burst 6.0 / 3 = 60, and with burst 2.5 = 25
        let mut weapon = input(SOLID_SHOT, 30, 50, 10, 0.0);
        weapon.weapon_warhead = CONVENTIONAL;
        weapon.burst_duration = 6.0;
        assert_eq!(forecast(&weapon).unwrap().capability, 60);
        weapon.burst_duration = 2.5;
        assert_eq!(forecast(&weapon).unwrap().capability, 25);
        // the same figures on a weapon that is not conventional munitions do not use the burst
        weapon.weapon_warhead = SOLID_SHOT;
        assert_eq!(forecast(&weapon).unwrap().capability, 30);
    }

    #[test]
    fn a_player_target_is_scaled_by_the_players_setting_after_the_raw_capability() {
        let mut target = input(SOLID_SHOT, 100, 50, 10, 0.0);
        target.not_ai = true;
        for (setting, expected) in [(0.33_f32, 33), (0.66, 66), (1.0, 100)] {
            target.player_modifier = setting;
            let outcome = forecast(&target).unwrap();
            assert_eq!((outcome.capability, outcome.raw), (expected, 100), "setting {setting}");
        }
        // the truncation is towards zero: 50 * 0.33 = 16.5
        let mut half = input(HE_PLAIN, 100, 1, 4, 0.5);
        half.not_ai = true;
        half.player_modifier = 0.33;
        let outcome = forecast(&half).unwrap();
        assert_eq!((outcome.capability, outcome.raw), (16, 50));
        // an AI target ignores whatever setting it was sent
        let mut ai = input(SOLID_SHOT, 100, 50, 10, 0.0);
        ai.player_modifier = 0.33;
        assert_eq!(forecast(&ai).unwrap().capability, 100);
    }

    #[test]
    fn a_question_the_original_has_no_answer_for_is_refused() {
        assert!(forecast(&input(SOLID_SHOT, 12, 20, -1, 0.0)).unwrap_err().contains("below zero"));
        let mut bad = input(SOLID_SHOT, 12, 20, 10, 0.0);
        bad.velocity = f32::NAN;
        assert!(forecast(&bad).unwrap_err().contains("finite"));
        bad = input(SOLID_SHOT, 12, 20, 10, f32::INFINITY);
        assert!(forecast(&bad).unwrap_err().contains("finite"));
        // a damage figure so large that the product does not fit an integer
        assert!(forecast(&input(SOLID_SHOT, i32::MAX, 20, 10, 0.0)).unwrap_err().contains("not an integer"));
    }
}
