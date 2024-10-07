use std::fs::{OpenOptions, File};
use std::io::{self, Write, BufRead};
use std::path::Path;
use std::sync::{Arc, RwLock, atomic::{AtomicU32, Ordering}};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use num_cpus;

fn is_prime(n: u32, primes: &[u32]) -> bool {
    if n < 2 {
        return false;
    }
    let sqrt_n = (n as f64).sqrt() as u32;
    for &p in primes.iter() {
        if p > sqrt_n {
            break;
        }
        if n % p == 0 {
            return false;
        }
    }
    true
}

fn load_primes_from_file(path: &str) -> Vec<u32> {
    if !Path::new(path).exists() {
        return Vec::new();
    }

    let file = File::open(path).expect("Не удалось открыть файл");
    let reader = std::io::BufReader::new(file);
    let mut primes = Vec::new();

    for line in reader.lines() {
        if let Ok(num) = line.expect("Не удалось прочитать строку").trim().parse::<u32>() {
            primes.push(num);
        }
    }
    primes
}

fn save_primes_to_file(path: &str, primes: &[u32]) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .expect("Не удалось открыть файл для записи");

    for &prime in primes {
        writeln!(file, "{}", prime).expect("Не удалось записать простое число в файл");
    }
}

fn main() {
    let path_prefix = "primes"; // Префикс для файлов
    let mut file_count = 1; // Счетчик файлов
    let primes_per_file = 1_000_000; // Максимальное количество простых чисел в файле

    let mut path = format!("{}_{}.txt", path_prefix, format!("{:05}", file_count));
    let mut primes = load_primes_from_file(&path); // Загружаем простые числа из файла

    // Получаем количество ядер
    let num_cores = num_cpus::get();
    println!("Количество ядер в системе: {}", num_cores);

    // Следующее число для расчета простоты
    let next_number = if !primes.is_empty() {
        *primes.last().unwrap() + 1
    } else {
        2
    };

    // Оборачиваем вектор простых чисел в Arc
    let primes = Arc::new(Mutex::new(primes));
    let next_number = Arc::new(AtomicU32::new(next_number)); // Следующее число
    let total_count = Arc::new(Mutex::new(0)); // Общий счетчик найденных чисел
    let last_write = Arc::new(Mutex::new(Instant::now())); // Время последней записи
    let running = Arc::new(AtomicU32::new(1)); // Флаг для остановки

    // Поток для отслеживания нажатий клавиш
    let running_clone = Arc::clone(&running);
    thread::spawn(move || {
        let stdin = io::stdin();
        let _ = stdin.lock().lines().next(); // Ждем ввода
        println!("Остановка поиска...");
        running_clone.store(0, Ordering::SeqCst); // Устанавливаем флаг завершения
    });

    // Основной цикл поиска
    let mut handles = vec![];

    for i in 0..num_cores {
        let primes_clone = Arc::clone(&primes);
        let next_number_clone = Arc::clone(&next_number);
        let total_count_clone = Arc::clone(&total_count);
        let last_write_clone = Arc::clone(&last_write);
        let running_clone = Arc::clone(&running);
        let path_prefix_clone = path_prefix.to_string();
        let primes_per_file_clone = primes_per_file;

        let handle = thread::spawn(move || {
            println!("Поток {} запущен", i);
            let mut file_count = 1;
            let mut local_primes: Vec<u32> = Vec::new();

            loop {
                // Проверяем флаг завершения
                if running_clone.load(Ordering::SeqCst) == 0 {
                    println!("Поток {} завершает работу", i);
                    break;
                }

                let current_num = next_number_clone.fetch_add(1, Ordering::SeqCst); // Следующее число

                // Проверка на простоту с учетом ранее найденных простых чисел
                let primes_guard = primes_clone.lock().unwrap();
                if is_prime(current_num, &primes_guard) {
                    local_primes.push(current_num);

                    let mut count = total_count_clone.lock().unwrap();
                    *count += 1;

                    // Проверяем, если количество простых чисел в локальном буфере достигло 1 млн
                    if local_primes.len() >= primes_per_file_clone {
                        let path = format!("{}_{}.txt", path_prefix_clone, format!("{:05}", file_count));
                        save_primes_to_file(&path, &local_primes);
                        local_primes.clear(); // Очищаем локальный буфер
                        file_count += 1; // Увеличиваем номер файла
                    }

                    // Добавляем новое простое число в общий список
                    drop(primes_guard); // Освобождаем блокировку перед обновлением
                    let mut primes_lock = primes_clone.lock().unwrap();
                    primes_lock.push(current_num);
                }

                // Проверяем, прошла ли 1 минута для сохранения остатка чисел в файл
                if last_write_clone.lock().unwrap().elapsed() >= Duration::new(60, 0) {
                    let path = format!("{}_{}.txt", path_prefix_clone, format!("{:05}", file_count));
                    save_primes_to_file(&path, &local_primes); // Записываем текущие простые числа
                    local_primes.clear(); // Очищаем локальный буфер
                    *last_write_clone.lock().unwrap() = Instant::now(); // Обновляем время записи
                    file_count += 1; // Увеличиваем номер файла
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Поиск завершен. Найдено простых чисел: {}", *total_count.lock().unwrap());
}
