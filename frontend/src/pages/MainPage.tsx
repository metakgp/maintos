import { useEffect, useState } from "react";
import { Header } from "../components/Common/Common";
import { OAUTH_LOGIN_URL, useAuthContext } from "../utils/auth";
import { makeRequest } from "../utils/backend";

function MainPage() {
	const auth = useAuthContext();
	const [subtitle, setSubtitle] = useState<string | null>(null);

	const fetchProfile = async () => {
		const response = await makeRequest('profile', 'post');

		if (response.status == 'success') {
			setSubtitle(`Welcome, ${response.data.username}`);
		} else {
			setSubtitle("Error loading profile (username).");
		}
	}

	useEffect(() => {
		if (!auth.isAuthenticated) {
			window.location.assign(OAUTH_LOGIN_URL);
		} else {
			fetchProfile();
		}
	}, []);

	return <>
		<Header
			title="MetaKGP Maintainers' Dashboard"
			subtitle={subtitle ?? undefined}
		/>
	</>;
}

export default MainPage;