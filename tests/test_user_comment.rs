extern crate server_v2;


mod user_comment_test {
    use futures_await_test::async_test;
    use mongodb::bson::doc;
    use rand::Rng;
    use uuid::Uuid;

    use server_v2::resources::register_link::get_register_link;
    use server_v2::resources::session::{AuthInfo, post_session, Session};
    use server_v2::resources::session::SESSION_POOL;
    use server_v2::resources::user::{post_user, RegisterInfo};
    use server_v2::util::database::DEFAULT_DATABASE;

    async fn create_user() -> AuthInfo {
        let username = Uuid::new_v4().to_string();
        let mut rng = rand::thread_rng();
        let email = (rng.gen_range(1000_0000, 9999_9999) as u32).to_string();
        let code = get_register_link(&(email.clone() + "@sustech.edu.cn")).await.unwrap().code;

        assert!(post_user(None, RegisterInfo{username:username.clone(), password: "test".to_string(), vcode: code, email}).await.is_ok());
        AuthInfo {
            username,
            password: "test".to_string(),
        }
    }

    async fn delete_user(username: &str) {
        let db = &DEFAULT_DATABASE;
        assert!(db.cli.database(&db.name).collection("User")
            .delete_one(doc! {"username": username}, None).await.is_ok());
    }

    async fn login(auth: AuthInfo) -> Session {
        let res = post_session(auth).await;
        assert!(res.is_ok());
        res.unwrap()
    }

    #[async_test]
    async fn test_post_user_delete_user() {
        let auth = create_user().await;
        let username = auth.username.clone();
        let s1 = login(auth).await;
        let s2 = SESSION_POOL.lock().unwrap().get(&s1.token).unwrap().clone();
        assert_eq!(s1, s2);
        delete_user(&username).await;
    }
}