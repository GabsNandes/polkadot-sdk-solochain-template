use crate::{mock::*, Error, Event};

    use frame_support::{assert_noop, assert_ok, BoundedVec};
	use frame_support::traits::ConstU32;

    // Helper function to create a bounded vec from a string
	fn bounded_vec(s: &str) -> BoundedVec<u8, ConstU32<256>> {
        BoundedVec::try_from(s.as_bytes().to_vec()).unwrap()
    }

    fn bounded_tweet(s: &str) -> BoundedVec<u8, ConstU32<280>> {
        BoundedVec::try_from(s.as_bytes().to_vec()).unwrap()
    }

#[test]
fn create_verify_user_test() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.

		assert_ok!(TemplateModule::create_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			1990, 5, 4 ));
		
		assert_noop!(TemplateModule::create_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			1990, 5, 4 ),
		Error::<Test>::NameAlreadyTaken);

		assert_noop!(TemplateModule::create_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Greg"), 
			bounded_vec("123"), 
			1800, 5, 4 ),
		Error::<Test>::InvalidBirthday);

		// Read pallet storage and assert an expected result.
		// Assert that the correct event was deposited
		System::assert_has_event(Event::NameStored { name: bounded_vec("Alice"), who: 1 }.into());
		
		assert_ok!(TemplateModule::verify_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123")));
		
		assert_noop!(TemplateModule::verify_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("errado")),
		Error::<Test>::InvalidCredentials
	);
	}
	)
}

#[test]
fn create_tweet_test() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TemplateModule::create_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			1990, 5, 4 ));

		assert_ok!(TemplateModule::create_tweet(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			bounded_tweet("MemeBigTest")));

		// Read pallet storage and assert an expected result.
		// Assert that the correct event was deposited
		System::assert_has_event(Event::NameStored { name: bounded_vec("Alice"), who: 1 }.into());
		System::assert_has_event(Event::BirthdayStored { who: 1, year: 1990, month: 5, day: 4 }.into());
		System::assert_has_event(Event::TweetCreated { who: 1, name: bounded_vec("Alice"), tweet_id: 0, timestamp: 0 }.into());
		
		assert_noop!(
			TemplateModule::create_tweet(
				RuntimeOrigin::signed(1),
				bounded_vec("Alice"),
				bounded_vec("wrongpassword"),
				bounded_tweet("Hello, World!")
			),
			Error::<Test>::InvalidCredentials
		);


	}
	)
}


#[test]
fn update_user_test() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.

		assert_ok!(TemplateModule::create_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			1990, 5, 4 ));

		// Read pallet storage and assert an expected result.
		// Assert that the correct event was deposited
		System::assert_has_event(Event::NameStored { name: bounded_vec("Alice"), who: 1 }.into());
		
		assert_ok!(TemplateModule::update_user_name(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"),
			bounded_vec("AliceNewName")
		));
		
		assert_noop!(TemplateModule::verify_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123")),
		Error::<Test>::UserNotFound
		);

		assert_ok!(TemplateModule::update_password(
			RuntimeOrigin::signed(1), 
			bounded_vec("AliceNewName"), 
			bounded_vec("123"),
			bounded_vec("New123")
		));

		assert_noop!(TemplateModule::verify_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("AliceNewName"), 
			bounded_vec("123")),
		Error::<Test>::InvalidCredentials
		);

		assert_ok!(TemplateModule::verify_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("AliceNewName"), 
			bounded_vec("New123")));
	}
	)
}

#[test]
fn delete_user_test() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TemplateModule::create_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			1990, 5, 4 ));

		assert_ok!(TemplateModule::delete_user(
		RuntimeOrigin::signed(1), 
		bounded_vec("Alice"), 
		bounded_vec("123") ));

		assert_noop!(
			TemplateModule::verify_user(
				RuntimeOrigin::signed(1),
				bounded_vec("alice"),
				bounded_vec("password123")
			),
			Error::<Test>::UserNotFound
		);



	}
	)
}


#[test]
fn delete_tweet_test() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(TemplateModule::create_user(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			1990, 5, 4 ));

		assert_ok!(TemplateModule::create_tweet(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			bounded_vec("123"), 
			bounded_tweet("MemeBigTest")));

		// Read pallet storage and assert an expected result.
		// Assert that the correct event was deposited
		System::assert_has_event(Event::NameStored { name: bounded_vec("Alice"), who: 1 }.into());
		System::assert_has_event(Event::BirthdayStored { who: 1, year: 1990, month: 5, day: 4 }.into());
		System::assert_has_event(Event::TweetCreated { who: 1, name: bounded_vec("Alice"), tweet_id: 0, timestamp: 0 }.into());
		
		assert_ok!(TemplateModule::delete_tweet(
			RuntimeOrigin::signed(1), 
			bounded_vec("Alice"), 
			0,
			bounded_vec("123"), 
			));

			assert_noop!(
                TemplateModule::delete_tweet(
                    RuntimeOrigin::signed(1),
                    bounded_vec("Alice"),
                    0,
                    bounded_vec("123")
                ),
                Error::<Test>::NoneValue
            );


	}
	)
}


