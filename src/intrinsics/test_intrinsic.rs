#[cfg(test)]
mod test_intrinsic {
    use std::env;

    use crate::intrinsics::INTRINSICS;
    use crate::{find_intrinsic, Intrinsic};

    #[test]
    fn test_find_intrinsic() {
        for intrinsic in INTRINSICS.iter() {
            let cmd_string = intrinsic.command.to_string();
            let found = find_intrinsic(&cmd_string);
            assert!(found.is_some());
            assert_eq!(found.unwrap(), intrinsic);
        }
    }

    #[test]
    fn test_intrinsic_handler_cd() {
        let command = "cd".to_string();
        let intrinsic = find_intrinsic(&command).unwrap();

        // Test empty argument
        let mut args = vec![];
        let mut result = (intrinsic.handler)(&args);
        assert!(result.is_ok());
        assert_eq!(
            env::current_dir().unwrap().to_str().unwrap(),
            env::var("HOME").unwrap().as_str()
        );

        // Test argument with absolute path
        args = vec!["/".to_string()];
        result = (intrinsic.handler)(&args);
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap().to_str().unwrap(), "/".to_string());

        args = vec!["~".to_string()];
        result = (intrinsic.handler)(&args);
        assert!(result.is_ok());
        assert_eq!(
            env::current_dir().unwrap().to_str().unwrap(),
            env::var("HOME").unwrap().as_str()
        );

        // Test argument with relative path
        let initial_dir = env::current_dir().unwrap();
        args = vec![".".to_string()];
        result =(intrinsic.handler)(&args);
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), initial_dir);

        args = vec!["..".to_string()];
        result =(intrinsic.handler)(&args);
        assert!(result.is_ok());
        assert_eq!(env::current_dir().unwrap(), initial_dir.parent().unwrap());
    }
}
