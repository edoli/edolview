#![allow(clippy::upper_case_acronyms)] // Metric names intentionally use their conventional abbreviations.

use std::{
    collections::HashSet,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
};

use super::{gpu_compute, Image, ImageData, Recti};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum StatisticsType {
    MinMax,
    Std,
    MSE,
    MAE,
    PSNRRMSE,
    SSIM,
    MSSSIM,
    FSIM,
}

impl std::fmt::Display for StatisticsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            StatisticsType::MinMax => "Min/Max/Variance/Std Dev",
            StatisticsType::Std => "Std Dev",
            StatisticsType::MSE => "MSE",
            StatisticsType::MAE => "MAE",
            StatisticsType::PSNRRMSE => "PSNR/MSE",
            StatisticsType::SSIM => "SSIM",
            StatisticsType::MSSSIM => "MSSSIM",
            StatisticsType::FSIM => "FSIM",
        };
        write!(f, "{s}")
    }
}

pub struct StatisticsResult {
    pub stat_type: StatisticsType,
    pub value: Vec<f64>,
    pub scope: StatisticsScope,
    pub error: Option<String>,
}

pub struct StatisticsUpdate {
    pub stat_type: StatisticsType,
    pub value: Vec<f64>,
    pub is_pending: bool,
    pub scope: StatisticsScope,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatisticsScope {
    pub request_id: u64,
    pub rect: Recti,
    pub asset_hash: Option<String>,
}

#[derive(Default)]
pub struct ValueWithScope<T> {
    pub value: T,
    pub scope: Option<StatisticsScope>,
}

#[derive(Default)]
pub struct MinMax {
    pub min: Vec<f64>,
    pub max: Vec<f64>,
    pub variance: Vec<f64>,
    pub std_dev: Vec<f64>,
}

#[derive(Default)]
pub struct PSNRRMSE {
    pub psnr: f64,
    pub rmse: f64,
}

#[derive(Default)]
pub struct Statistics {
    pub psnr_rmse: ValueWithScope<PSNRRMSE>,
    pub ssim: ValueWithScope<f64>,
    pub min_max: ValueWithScope<MinMax>,
}

pub struct StatisticsWorker {
    tx: Sender<StatisticsResult>,
    rx: Receiver<StatisticsResult>,

    processing: HashSet<StatisticsType>,
    pending: HashSet<StatisticsType>,
}

impl StatisticsWorker {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tx,
            rx,
            processing: HashSet::new(),
            pending: HashSet::new(),
        }
    }

    pub fn run_statistics(&mut self, image: ImageData, scale: f64, scope: StatisticsScope) {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Statistics::ImageWrapper");

        if image.spec().width <= 0 || image.spec().height <= 0 {
            return;
        }

        let result_size = image.spec().channels as usize * 4;
        self.run(StatisticsType::MinMax, scope, result_size, move |scope| {
            #[cfg(debug_assertions)]
            let _timer = crate::util::timer::ScopedTimer::new("Statistics::Image");

            let texture = image.gpu_texture()?;
            let statistics = gpu_compute()?.statistics(&texture, scope.rect)?;
            Ok::<Vec<f64>, color_eyre::Report>(
                (0..statistics.min.len())
                    .flat_map(|c| {
                        [
                            statistics.min[c] * scale,
                            statistics.max[c] * scale,
                            statistics.variance[c] * scale * scale,
                            statistics.std_dev[c] * scale.abs(),
                        ]
                    })
                    .collect(),
            )
        });
    }

    pub fn run_psnr(
        &mut self,
        image1: ImageData,
        image2: ImageData,
        data_range: f64,
        scale: f64,
        scope: StatisticsScope,
    ) {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Statistics::PSNRWrapper");

        if image1.spec().width <= 0 || image2.spec().width <= 0 {
            return;
        }

        self.run(StatisticsType::PSNRRMSE, scope, 2, move |scope| {
            #[cfg(debug_assertions)]
            let _timer = crate::util::timer::ScopedTimer::new("Statistics::PSNR");

            let texture1 = image1.gpu_texture()?;
            let texture2 = image2.gpu_texture()?;
            let (psnr, rmse) = gpu_compute()?.psnr(&texture1, &texture2, scope.rect, data_range, scale)?;
            Ok::<Vec<f64>, color_eyre::Report>(vec![psnr, rmse])
        });
    }

    pub fn run_ssim(&mut self, image1: ImageData, image2: ImageData, scope: StatisticsScope) {
        #[cfg(debug_assertions)]
        let _timer = crate::util::timer::ScopedTimer::new("Statistics::SSIMWrapper");

        if image1.spec().width <= 0 || image2.spec().width <= 0 {
            return;
        }

        self.run(StatisticsType::SSIM, scope, 1, move |scope| {
            #[cfg(debug_assertions)]
            let _timer = crate::util::timer::ScopedTimer::new("Statistics::SSIM");

            let texture1 = image1.gpu_texture()?;
            let texture2 = image2.gpu_texture()?;
            Ok::<Vec<f64>, color_eyre::Report>(vec![gpu_compute()?.ssim(&texture1, &texture2, scope.rect)?])
        });
    }

    pub fn run<F, E>(&mut self, stat_type: StatisticsType, scope: StatisticsScope, result_size: usize, func: F)
    where
        F: FnOnce(&StatisticsScope) -> Result<Vec<f64>, E> + Send + 'static,
        E: std::fmt::Debug,
    {
        if self.processing.contains(&stat_type) {
            self.pending.insert(stat_type);
            return;
        } else {
            self.pending.remove(&stat_type);
        }

        self.processing.insert(stat_type.clone());

        let tx = self.tx.clone();
        thread::spawn(move || {
            match func(&scope) {
                Ok(val) => {
                    let _ = tx.send(StatisticsResult {
                        stat_type,
                        value: val,
                        scope,
                        error: None,
                    });
                }
                Err(e) => {
                    let error = format!("Error computing {stat_type}: {e:?}");
                    eprintln!("StatisticsWorker: {error}");
                    let _ = tx.send(StatisticsResult {
                        stat_type,
                        value: vec![f64::NAN; result_size],
                        scope,
                        error: Some(error),
                    });
                }
            };
        });
    }

    pub fn invalidate(&mut self) -> Vec<StatisticsUpdate> {
        let mut invalidated = Vec::new();

        loop {
            match self.rx.try_recv() {
                Ok(msg) => {
                    self.processing.remove(&msg.stat_type);

                    // When statistics in calculated while another request is pending,
                    // mark it as pending again
                    let is_pending = self.pending.contains(&msg.stat_type);

                    invalidated.push(StatisticsUpdate {
                        stat_type: msg.stat_type,
                        value: msg.value,
                        is_pending,
                        scope: msg.scope,
                        error: msg.error,
                    });
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(_) => {
                    // Disconnected
                    eprintln!("StatisticsWorker channel disconnected");
                    break;
                }
            }
        }
        invalidated
    }
}
