use std::io;

fn main() {
    let mut input = String::new();

    // Вводим размер двумерного вектора (n - количество строк, m - количество столбцов)
    println!("Введите количество строк и столбцов:");
    io::stdin().read_line(&mut input).expect("Не удалось прочитать строку");

    let dims: Vec<usize> = input
        .trim()
        .split_whitespace()
        .map(|s| s.parse().expect("Введите целое число"))
        .collect();
    
    let (n, m) = (dims[0], dims[1]);  // n - строки, m - столбцы

    // Создаем двумерный вектор (n строк по m элементов)
    let mut matrix: Vec<Vec<i32>> = vec![vec![0; m]; n];

    println!("Введите элементы матрицы ({} чисел):", n * m);
    let mut counter = 1; // Начинаем счетчик с 1

    // Заполняем матрицу значениями
    for i in 0..n {
        for j in 0..m {
            matrix[i][j] = counter;
            counter += 1; // Увеличиваем счетчик на 1
        }
    }

    // Выводим матрицу
    println!("Матрица:");
    for row in &matrix {
        for elem in row {
            print!("{} ", elem);
        }
        println!();
    }
}