import React, {
	createContext,
	useContext,
	useEffect,
	useMemo,
	useState,
} from "react";
import { useNavigate } from "react-router-dom";
import { makeRequest } from "./backend";

interface IAuthContext {
	isAuthenticated: boolean;
	jwt: string | null;
	username: string | null;
	login: (jwt: string) => void;
	logout: () => void;
}

const DEFAULT_AUTH_CONTEXT: IAuthContext = {
	isAuthenticated: false,
	jwt: null,
	username: null,
	login: () => { },
	logout: () => { }
};

const getLsAuthJwt = () => {
	return localStorage.getItem("jwt");
};

const AuthContext = createContext<IAuthContext>(DEFAULT_AUTH_CONTEXT);

export const useAuthContext = () => useContext(AuthContext);

export function AuthProvider({ children }: { children: React.ReactNode }) {
	const navigate = useNavigate();

	const lsAuthJwt = getLsAuthJwt();
	const [isAuthenticated, setIsAuthenticated] = useState(
		lsAuthJwt !== null && lsAuthJwt !== "",
	);
	const [username, setUsername] = useState<string | null>(null);

	const login = async (jwt: string) => {
		localStorage.setItem("jwt", jwt);
		await checkAuth(jwt);
	};

	const logout = () => {
		localStorage.removeItem("jwt");
		setIsAuthenticated(false);
		setUsername(null);
		navigate('/');
	};

	const checkAuth = async (jwt: string) => {
		const response = await makeRequest('profile', 'get', null, jwt);

		if (response.status !== 'success') {
			localStorage.removeItem("jwt");
			setIsAuthenticated(false);
		} else {
			setUsername(response.data.username);
			setIsAuthenticated(true);
		}
	}
	useEffect(() => {
		if (isAuthenticated) {
			checkAuth(lsAuthJwt as string);
		}
	}, [])

	const value = useMemo(
		() => ({
			isAuthenticated,
			jwt: lsAuthJwt,
			username: username,
			login,
			logout
		}),
		[isAuthenticated, username, login, logout],
	);

	return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export const OAUTH_LOGIN_URL = `https://github.com/login/oauth/authorize?client_id=${import.meta.env.VITE_GH_OAUTH_CLIENT_ID}`;
