use std::fmt::Debug;

use super::{is_subset, C3Error, Sets, C3};

// TODO: Re-implement using single Classes.
pub fn c3_linearization(mut input: C3) -> Result<C3, C3Error> {
    let mut output = C3::new();
    loop {
        let solved = output.all_classes();
        let mut found = false;
        for base in input.all_classes() {
            let parents = input.path(&base)?;
            if is_subset(&solved, &parents) {
                let sets = output.sets_for(parents)?;
                let solution = merge(&base, sets)?;
                input.remove(&base)?;
                output.add(base, solution);
                found = true;
            }
        }
        if !found {
            break;
        }
    }
    Ok(output)
}

pub fn merge<T: Clone + Debug + Eq>(base: &T, mut sets: Sets<T>) -> Result<Vec<T>, C3Error> {
    let mut solutions = vec![base.clone()];
    loop {
        if sets.is_empty() {
            return Ok(solutions);
        }
        let solution = sets.find_solution()?;
        solutions.push(solution);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c3() {
        let mut input = C3::new();
        input.add_class_str("A", "");
        input.add_class_str("B", "A");
        input.add_class_str("C", "A");
        input.add_class_str("D", "B, C");

        let mut target = C3::new();
        target.add_class_str("A", "A");
        target.add_class_str("B", "B, A");
        target.add_class_str("C", "C, A");
        target.add_class_str("D", "D, B, C, A");

        assert_eq!(c3_linearization(input).unwrap(), target);
    }

    #[test]
    fn test_c3_second_example() {
        let mut input = C3::new();
        input.add_class_str("Context", "");
        input.add_class_str("IERC20", "");
        input.add_class_str("IERC20Metadata", "IERC20");
        input.add_class_str("IERC20Errors", "");
        input.add_class_str("ERC20", "Context, IERC20Metadata, IERC20, IERC20Errors");
        input.add_class_str("ERC20Burnable", "ERC20, Context");
        input.add_class_str("ERC20Capped", "ERC20");
        input.add_class_str("Ownable", "Context");
        input.add_class_str("Plascoin", "ERC20Capped, ERC20Burnable, Ownable");

        let mut target = C3::new();
        target.add_class_str("Context", "Context");
        target.add_class_str("IERC20", "IERC20");
        target.add_class_str("IERC20Metadata", "IERC20Metadata, IERC20");
        target.add_class_str("IERC20Errors", "IERC20Errors");
        target.add_class_str(
            "ERC20",
            "ERC20, Context, IERC20Metadata, IERC20, IERC20Errors",
        );
        target.add_class_str(
            "ERC20Burnable",
            "ERC20Burnable, ERC20, Context, IERC20Metadata, IERC20, IERC20Errors",
        );
        target.add_class_str(
            "ERC20Capped",
            "ERC20Capped, ERC20, Context, IERC20Metadata, IERC20, IERC20Errors",
        );
        target.add_class_str("Ownable", "Ownable, Context");
        target.add_class_str("Plascoin", "Plascoin, ERC20Capped, ERC20Burnable, ERC20, Ownable, Context, IERC20Metadata, IERC20, IERC20Errors");

        assert_eq!(c3_linearization(input).unwrap(), target);
    }

    #[test]
    fn test_merge() {
        let head = "K";
        let mut sets: Sets<&str> = Sets::new();
        sets.push(vec!["B", "O"]).unwrap();
        sets.push(vec!["A", "O"]).unwrap();
        sets.push(vec!["C", "O"]).unwrap();
        sets.push(vec!["B", "A", "C"]).unwrap();

        let result = merge(&head, sets).unwrap();
        assert_eq!(result, vec!["K", "B", "A", "C", "O"]);
    }

    #[test]
    fn test_merge_2() {
        let head = "Z";
        let mut sets: Sets<&str> = Sets::new();
        sets.push(vec!["K1", "C", "B", "A", "O"]).unwrap();
        sets.push(vec!["K3", "A", "D", "O"]).unwrap();
        sets.push(vec!["K2", "B", "D", "E", "O"]).unwrap();
        sets.push(vec!["K1", "K3", "K2"]).unwrap();

        let result = merge(&head, sets).unwrap();
        let expected = vec!["Z", "K1", "C", "K3", "K2", "B", "A", "D", "E", "O"];
        assert_eq!(result, expected);
    }
}
