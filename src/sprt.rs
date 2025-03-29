use core::f32;

#[derive(Debug, Clone)]
pub struct SPRT {
    elo0: u32,
    elo1: u32,
    alpha: f32,
    beta: f32,
    upper: f32,
    lower: f32,
}

pub enum SPRTResult {
    AcceptH1,
    AcceptH0,
    Continue,
}

pub struct SPRTStatus {
    pub result: SPRTResult,
    pub llr: f32,
}

impl SPRT {
    pub fn new(elo0: u32, elo1: u32, alpha: f32, beta: f32) -> Self {
        let lower = f32::ln(beta / (1.0 - alpha));
        let upper = f32::ln((1.0 - beta) / alpha);

        Self {
            elo0,
            elo1,
            alpha,
            beta,

            lower,
            upper,
        }
    }

    pub fn status(&self, wins: u32, losses: u32, draws: u32) -> SPRTStatus {
        let llr = sprt(
            wins as f32,
            losses as f32,
            draws as f32,
            self.elo0 as f32,
            self.elo1 as f32,
        );

        let result = if llr >= self.upper {
            SPRTResult::AcceptH1
        } else if llr <= self.lower {
            SPRTResult::AcceptH0
        } else {
            SPRTResult::Continue
        };

        SPRTStatus { result, llr }
    }

    pub fn elo0(&self) -> u32 {
        self.elo0
    }

    pub fn elo1(&self) -> u32 {
        self.elo1
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn beta(&self) -> f32 {
        self.beta
    }

    pub fn upper(&self) -> f32 {
        self.upper
    }

    pub fn lower(&self) -> f32 {
        self.lower
    }
}

struct Probability {
    win: f32,
    loss: f32,
    draw: f32,
}

struct BayesElo {
    elo: f32,
    draw: f32,
}

fn adj_probs(b: BayesElo) -> Probability {
    let win = expected_score(-b.draw + b.elo);
    let loss = expected_score(-b.draw - b.elo);

    Probability {
        win,
        loss,
        draw: 1.0 - win - loss,
    }
}

// fn scale(draw_elo: f32) -> f32 {
//     let x = f32::powf(10.0, -draw_elo / 400.0);
//     4.0 * x / f32::powi(1.0 + x, 2)
// }

fn expected_score(x: f32) -> f32 {
    1.0 / (1.0 + f32::powf(10.0, -x / 400.0))
}

fn sprt(wins: f32, losses: f32, draws: f32, elo0: f32, elo1: f32) -> f32 {
    if wins == 0.0 || losses == 0.0 {
        return 0.0;
    }

    let total = wins + draws + losses;

    let probs = Probability {
        win: wins / total,
        loss: losses / total,
        draw: draws / total,
    };

    let draw_elo = 200.0 * f32::log10((1.0 - 1.0 / probs.win) * (1.0 - 1.0 / probs.loss));
    // cutechess would scale elo0 and elo1 by the draw_elo
    let s = 1.0;

    let b0 = BayesElo {
        elo: elo0 / s,
        draw: draw_elo,
    };

    let b1 = BayesElo {
        elo: elo1 / s,
        draw: draw_elo,
    };

    let p0 = adj_probs(b0);
    let p1 = adj_probs(b1);

    wins * f32::ln(p1.win / p0.win)
        + losses * f32::ln(p1.loss / p0.loss)
        + draws * f32::ln(p1.draw / p0.draw)
}

fn erf_inv(x: f32) -> f32 {
    use f32::consts::PI;

    let a = 8.0 * (PI - 3.0) / (3.0 * PI * (4.0 - PI));
    let y = f32::ln(1.0 - x * x);
    let z = 2.0 / (PI * a) + y / 2.0;

    f32::sqrt(f32::sqrt(z * z - y / a) - z).copysign(x)
}

fn phi_inv(p: f32) -> f32 {
    use f32::consts::SQRT_2;

    SQRT_2 * erf_inv(2.0 * p - 1.0)
}

fn elo(score: f32) -> f32 {
    if score <= 0.0 || score >= 1.0 {
        return 0.0;
    }

    -400.0 * f32::log10(1.0 / score - 1.0)
}

pub fn elo_wld(wins: u32, losses: u32, draws: u32) -> (f32, f32, f32) {
    let num_games = wins + losses + draws;

    let p_w = wins as f32 / num_games as f32;
    let p_l = losses as f32 / num_games as f32;
    let p_d = draws as f32 / num_games as f32;

    let mu = p_w + p_d / 2.0;

    let dev_w = p_w * f32::powi(1.0 - mu, 2);
    let dev_l = p_l * f32::powi(0.0 - mu, 2);
    let dev_d = p_d * f32::powi(0.5 - mu, 2);

    let stdev = f32::sqrt(dev_w + dev_l + dev_d) / f32::sqrt(num_games as f32);

    let mu_min = mu + phi_inv(0.025) * stdev;
    let mu_max = mu + phi_inv(0.975) * stdev;

    (elo(mu_min), elo(mu), elo(mu_max))
}
