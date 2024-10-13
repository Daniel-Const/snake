use std::{
    collections::VecDeque,
    thread,
    time,
    time::Duration,
    io::{
        stdout,
        Stdout,
        Write
    }
};
use rand::Rng;
use termion::{
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
    async_stdin,
    AsyncReader,
};

const SNAKE_CHAR: char = '■';
const BG_CHAR: char = '.';
const FRUIT_CHAR: char = 'x';
const FRAME_DELAY: Duration = time::Duration::from_millis(80);

#[derive(Clone, Copy)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

struct Snake {
    size: usize,
    direction: Direction,
    positions: VecDeque<(usize, usize)>,
    grow: usize,
}

impl Snake {
    fn new(board: &Board) -> Snake {
        let x: usize = board.width / 2;
        let y: usize = board.height / 2;
        return Snake {
            size: 2,
            direction: Direction::DOWN,
            positions: VecDeque::from([(x, y), (x, y-1)]),
            grow: 0
        }
    }

    fn grow(&mut self) {
        self.grow += 1;
    }

    fn move_position(&mut self, board: &Board) -> (usize, usize) {
        let tail_pos: (usize, usize);
        // Get the tail position
        if let Some(tail) = self.positions.front() {
            tail_pos = *tail;
        } else {
            tail_pos = (0, 0); // Handle better?
        }
        
        // If still growing do not remove tail from current position
        if self.grow > 0 {
            self.grow -= 1; 
        } else {
            self.positions.remove(0);
        }

        if let Some((mut x, mut y)) = self.positions.back() {
            match self.direction {
                Direction::UP => {
                    if y != 0 {
                        y -= 1;
                    } else {
                        y = board.height - 1;
                    }
                }
                Direction::DOWN => {
                    y += 1;
                    if y >= board.height {
                        y = 0;
                    }
                },
                Direction::LEFT => {
                    if x != 0 {
                        x -= 1;
                    } else {
                        x = board.width - 1;
                    }
                },
                Direction::RIGHT => {
                    x += 1;
                    if x >= board.width {
                        x = 0;
                    }
                },
            }
           self.positions.push_back((x, y));
        }

        return tail_pos;
    }
}

struct Board {
    height: usize,
    width: usize,
    cells: Vec<Vec<char>>,
}

impl Board {
    fn new(height: usize, width: usize) -> Board {
        let board = Board {
            height,
            width,
            cells: vec![vec![BG_CHAR; width]; height]
        };

        return board;
    }

    fn draw_snake(&mut self, snake: &Snake, old_position: (usize, usize)) {
        let (x, y) = old_position;
        self.cells[y][x] = BG_CHAR;
        for position in snake.positions.iter() {
            let (x, y) = position;
            self.cells[*y][*x] = SNAKE_CHAR;
        }
    }

    fn draw_fruit(&mut self, fruit_position: (usize, usize)) {
        let (x, y) = fruit_position;
        self.cells[y][x] = FRUIT_CHAR;
    }
}

struct Game {
    board: Board,
    snake: Snake,
    fruit_position: (usize, usize)
}

impl Game {
    fn print(&self, mut stdout: &Stdout) {
        // Control char: Clear terminal output
        write!(
            stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::Hide
        )
        .unwrap();
        stdout.flush().unwrap();

        for row in 0..self.board.height {
            write!(stdout, "\r").unwrap();
            let line: String = self.board.cells[row].iter().map(|ch| format!("{ch} ", ch=ch)).collect();
            write!(stdout, "{line}\n").unwrap();
        }

        write!(stdout, "\r\nq to exit; Control with arrow keys").unwrap();

        stdout.flush().unwrap();
    }



    fn step(&mut self) {
        /* Move the snakes position and update the board */
        let old_pos = self.snake.move_position(&self.board);
        self.board.draw_snake(&self.snake, old_pos);
        self.check_fruit();
        self.board.draw_fruit(self.fruit_position);
    }

    fn keyboard_action(&mut self, key: termion::event::Key) {
        match key {
            Key::Up => {
                self.snake.direction = Direction::UP;
            },
            Key::Down => {
                self.snake.direction = Direction::DOWN;
            }, 
            Key::Right => {
                self.snake.direction = Direction::RIGHT;
            },
            Key::Left => {
                self.snake.direction = Direction::LEFT;
            },
            Key::Char('q') => std::process::exit(0x0100),
            _ => ()
        }
    }

    fn new_fruit_position(&mut self) {
        let mut rng = rand::thread_rng();
        let x: usize = rng.gen_range(0..self.board.width);
        let y: usize = rng.gen_range(0..self.board.height);
        self.fruit_position = (x, y);
    }

    fn check_fruit(&mut self) {
        // Check if fruit in snake positions
        if self.snake.positions.contains(&self.fruit_position) {
            self.new_fruit_position();
            self.snake.grow();
        }
    }

    fn init(&mut self) {
        self.new_fruit_position();
        self.board.draw_fruit(self.fruit_position);
        self.board.draw_snake(&self.snake, (0,0));
    }
}

fn main() {
    let board = Board::new(20, 20);
    let snake = Snake::new(&board);
    let mut game = Game {board, snake, fruit_position: (0,0)};
    game.init();

    let stdin: AsyncReader = async_stdin();
    let stdout = stdout().into_raw_mode().unwrap();
    let mut keys = stdin.keys();

    loop {
        let result = keys.next();
        match result {
            Some(key) => match key {
                Ok(k) => {
                    game.keyboard_action(k)
                },
                _ => {},
            },
            _ => (),
        }
        game.step();
        game.print(&stdout);
        thread::sleep(FRAME_DELAY);
    }
}
