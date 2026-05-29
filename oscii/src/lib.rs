const N: usize = 512;

pub struct WaveTable {
    table: [f32; N],
}

impl WaveTable {
    pub fn init_table(&mut self) {
        let step = 2.0 * std::f32::consts::PI / N as f32;
        for i in 0..N {
            self.table[i] = (i as f32 * step).sin();
        }
    }

    pub fn table_eval(&self, phase: f32) -> f32 {
        // x0 和 x1 是相邻的两个采样点，fr 是相位对应的索引
        let fr = phase * N as f32;
        let i = fr as usize; // 整数部分用于确定相邻的采样点
        let frac = fr - i as f32; // 小数部分用于线性插值

        let x0 = self.table[i % N];
        let x1 = self.table[(i + 1) % N];
        // 如果 N 是 2 的幂，则模运算可以用按位运算代替：
        // let x1 = self.table[(i+1) & (N-1)];

        (1.0 - frac) * x0 + frac * x1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_table() {
        let mut wave_table = WaveTable { table: [0.0; N] };
        wave_table.init_table();

        let threashold = 1e-6;
        // 测试第一个值是否为0
        assert!((wave_table.table[0] - 0.0).abs() < threashold);

        // 测试第128个值是否为1
        assert!((wave_table.table[128] - 1.0).abs() < threashold);

        // 测试第256个值是否为0
        assert!((wave_table.table[256] - 0.0).abs() < threashold);

        // 测试第384个值是否为-1
        assert!((wave_table.table[384] - (-1.0)).abs() < threashold);
    }

    #[test]
    fn test_table_eval() {
        let mut wave_table = WaveTable { table: [0.0; N] };
        wave_table.init_table();

        let threashold = 1e-6;
        // 测试相位为0时的输出值是否为0
        assert!((wave_table.table_eval(0.0) - 0.0).abs() < threashold);

        // 测试相位为0.25时的输出值是否为1
        assert!((wave_table.table_eval(0.25) - 1.0).abs() < threashold);

        // 测试相位为0.5时的输出值是否为0
        dbg!(wave_table.table_eval(0.5));
        assert!((wave_table.table_eval(0.5) - 0.0).abs() < threashold);

        // 测试相位为0.75时的输出值是否为-1
        assert!((wave_table.table_eval(0.75) - (-1.0)).abs() < threashold);
    }
}
