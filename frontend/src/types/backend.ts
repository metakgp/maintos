export type AllowedBackendMethods = "get" | "post";

export interface IOkResponse<T> {
	status: "success";
	message: string;
	status_code: 200;
	data: T;
}

export interface IErrorResponse {
	status: "error";
	message: string;
	status_code: number | string;
}

export type BackendResponse<T> = IOkResponse<T> | IErrorResponse;


export interface IEndpointTypes {
	oauth: {
		request: {
			code: string
		},
		response: {
			token: string
		}
	},
	profile: {
		request: null;
		response: {
			username: string;
			token: string;
		}
	},
}
