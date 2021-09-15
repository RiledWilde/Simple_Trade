use std::iter::FromIterator;

enum OrderType {
  Buy,
  Sell,
  Null,
}

struct Order {
  id: i32,
  order_type: OrderType,
  price: u32,
  quantity: u32
}

impl Order {

  fn out(&self) -> String
  {
    match self.order_type {
      OrderType::Buy => format!("{}: Buy {} BTC @ {}", self.id, self.price, self.quantity),
      OrderType::Sell => format!("{}: Sell {} BTC @ {}", self.id, self.price, self.quantity),
      _ => "-1".to_string(),
    }
  }

  fn new(s_in: &String) -> Order
  {
    let r_str = Vec::from_iter(s_in.split(" ").map(String::from));

    let mut tmp = String::new();
    if r_str[0].ends_with(":") || r_str[0].ends_with(".") {
      tmp = r_str[0][..r_str[0].len()-1].to_string();
    }

    let id_no = match tmp.parse::<i32>() {
      Ok(id_no) => id_no,
      Err(_) => panic!("ID not correct - contains more than numbers."),
    };
    
    let order = match r_str[1].to_ascii_lowercase().as_str() {
      "buy" => OrderType::Buy,
      "sell" => OrderType::Sell,
      _ => OrderType::Null,
    };

    let quantity = match r_str[2].to_string().parse::<u32>() {
      Ok(quantity) => quantity,
      Err(_) => 0,
    };
    
    let price = match r_str[5].to_string().parse::<u32>() {
      Ok(price) => price,
      Err(_) => 0,
    };

    Order {
      id : id_no,
      order_type : order,
      quantity : quantity,
      price : price,
    }
  }

}

struct Trade {
  buy_id : i32,
  sell_id : i32,
  quantity_traded : u32,
  price : u32
}

fn trimmer(s_in: &mut String) -> String
{
  if s_in.ends_with("\n") {
    s_in.pop();

    if s_in.ends_with("\r") {
      s_in.pop();
    }
  }

  s_in.to_string()
}

fn main()
{
  // Used for debugging purposes.
  use std::io::{self, prelude::*, BufReader};

  // Primitive order book.
  let mut buy_list : Vec<Order> = Vec::new();
  let mut sell_list : Vec<Order> = Vec::new();

  println!("Please enter your trades using stdin.");
  println!("The format should be of 'id: Buy/Sell quantity BTC @ price'.");

  {

    let args : Vec::<String> = std::env::args().skip(1).collect();
    //.. Extra arguments for quick testing from files.
    if args.len() > 0
    {
      let filename : String = std::env::args().nth(1).expect("We should have got an argument?");

      let fp = format!("{}.dat",&filename);
      let path = std::path::Path::new(&fp);

      let file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(_e) => panic!("Could not open the test file."),
      };

      let reader = BufReader::new(file);
      let lines = reader.lines();

      // Loop over each line in turn - like we have entered it one-by-one,
      // This is used for testing purposes.
      for l in lines
      {
        let mut line : String = l.unwrap();
        let mut t = trimmer(&mut line);
        t = t.to_ascii_lowercase();

        if !input_check(&t) {
          println!("What you have inputted does not match the requirements.");
          continue;
        }
        
        if t.contains(&"buy".to_string())
        {
          // New buy order.
          buy_list.push(Order::new(&t));
        }
        else if t.contains(&"sell".to_string())
        {
          // New sell order.
          sell_list.push(Order::new(&t));

          // Order the sell list.
          sell_list.sort_by(|x,y| x.price.cmp(&y.price));
        }

        // Call the trade routine which will loop over buy + sell list.
        if sell_list.len() > 0 && buy_list.len() > 0 {
          
          if trade_between(&mut buy_list, &mut sell_list) {
            buy_list.retain(|x| x.quantity != 0);
          }
        }
      }

    }
    else
    {
      // User input.
      println!("Press enter only to stop the program.");

      // Read in from stdin altogether - enter moves forward.
      loop {
        //println!("Please enter an order below.");

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        
        if "\n" != line {
          let mut t = trimmer(&mut line);
          t = t.to_ascii_lowercase();

          //..
          // Check the string just in case - can be changed at the end of the script.
          if !input_check(&t) {
            println!("What you have inputted does not match the requirements.");
            println!("Please try the same input again.");
            continue;
          }

          //..
          // Continue with the trade...
          if t.contains(&"buy".to_string())
          {
            buy_list.push(Order::new(&t));
          }
          else if t.contains(&"sell".to_string())
          {
            sell_list.push(Order::new(&t));
            // When a new sell order comes in we have to rearrange
            sell_list.sort_by(|x,y| x.price.cmp(&y.price));
          }

          // Call the trade routine which will loop over buy + sell list.
          if sell_list.len() > 0 && buy_list.len() > 0
          {
            if trade_between(&mut buy_list, &mut sell_list) {
              sell_list.retain(|x| x.quantity != 0);
            }
          }

        } else {
          println!("Final input submitted.");
          break;
        }
      }

    }
  }

  println!("Fin.");
}

fn trade_between(buy_list : &mut Vec<Order>, sell_list : &mut Vec<Order>) -> bool {

  let mut config : bool = false;

  for buy in buy_list.iter_mut() {
    for sell in sell_list.iter_mut() {

      //..
      // If the sell price is less than or equal to the buy price -> we trade.
      if sell.price <= buy.price && sell.quantity > 0 && buy.quantity > 0 {

        let quant = if sell.quantity <= buy.quantity {sell.quantity} else {buy.quantity};

        let trade_out : Trade = Trade {
          buy_id : buy.id,
          sell_id : sell.id,
          quantity_traded : quant,
          price : sell.price,
        };

        // Reduce the relevant quantities.
        if sell.quantity <= buy.quantity
        {
          buy.quantity = buy.quantity - sell.quantity;
          sell.quantity = 0;
        }
        else
        {
          sell.quantity = sell.quantity - buy.quantity;
          buy.quantity = 0;
        }

        println!("Trade {q} BTC @ {p} USD between {id_buy} and {id_sell}",
                  q=trade_out.quantity_traded, p=trade_out.price,
                  id_buy=trade_out.buy_id, id_sell=trade_out.sell_id);

        if config == false {
          config = true;
        }

        //..
        // Check if we have satisfied the condition.
        match buy.quantity {
          0 => break,
          _ => continue,
        };
      }
    }
    
    // Given case 4 - there was no other orders to fill up at the time so we must include this.
    // We set this to zero for the meantime so we can delete it afterward this routine.
    buy.quantity = 0;
  }

  config
}

fn input_check(s : &String) -> bool {

  let r_str = Vec::from_iter(s.split(" ").map(String::from));

  if r_str.len() != 7 {
    println!("Length of string is incorrect. Current length is {}.", r_str.len());
    return false;
  }

  {
    let tmp : String;

    if r_str[0].ends_with(":") || r_str[0].ends_with(".") {
      tmp = r_str[0][..r_str[0].len()-1].to_string();
    } else {
      tmp = r_str[0].to_string();
    }

    let are_num : Vec<bool> = tmp.chars().map(|x| x.is_numeric()).collect();
    if are_num.contains(&false) {
      println!("ID number contains non-digits {}.",tmp);
      return false;
    }
  }

  if !(r_str[1] == "buy" || r_str[1] == "sell") {
    println!("Buy or Sell not listed as an argument.");
    return false;
  }

  //..
  // Check you mean BTC and not another currencyhere..

  //..
  // Check @ symbol - to make sure correct thinking/input.

  {
    let are_num : Vec<bool> = r_str[5].chars().map(|x| x.is_numeric()).collect();
    if are_num.contains(&false) {
      println!("Amount is not numeric {}. Please check.", r_str[5]);
      return false;
    }
  }

  //.
  // Check correct currency here.

  true
}