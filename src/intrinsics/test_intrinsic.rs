#[cfg(test)]
mod test_intrinsic {
    use std::env;

    use crate::intrinsics::INTRINSICS;
    use crate::{find_intrinsic, Intrinsic};

    #[test]
    fn test_find_intrinsic() {
        for intrinsic in INTRINSICS.iter() {
            let found = find_intrinsic(&intrinsic.command.to_string());
            assert!(found.is_some());
            assert_eq!(found.unwrap(), intrinsic);
        }
    }

    #[test]
    fn test_intrinsic_handler_cd() {
        let intrinsic = find_intrinsic("cd".as_str()).unwrap();

        // Test empty argument
        let mut args = vec![];
        let result = intrinsic.handler(&args);
        assert!(result.is_ok());
        assert_eq!(
            env::current_dir().unwrap().to_str().unwrap(),
            env::var("HOME").unwrap().as_str()
        );

        // Test argument with absolute path
        args = vec!["/".to_string()];
        result = intrinsic.handler(&args);
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap().to_str().unwrap(), "/".as_str());

        args = vec!["~".to_string()];
        result = intrinsic.handler(&args);
        assert!(result.is_ok());
        assert_eq!(
            env::current_dir().unwrap().to_str().unwrap(),
            env::var("HOME").unwrap().as_str()
        );

        // Test argument with relative path
        let initial_dir = env::current_dir().unwrap();
        args = vec![".".to_string()];
        result = intrinsic.handler(&args);
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), initial_dir);

        args = vec!["..".to_string()];
        result = intrinsic.handler(&args);
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), initial_dir.parent().unwrap());
    }
}
