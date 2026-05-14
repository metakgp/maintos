import type { Deployment } from './deployments'

export type AllowedBackendMethods = 'get' | 'post'

export interface IOkResponse<T> {
	status: 'success'
	message: string
	status_code: 200
	data: T
}

export interface IErrorResponse {
	status: 'error'
	message: string
	status_code: number | string
}

export type BackendResponse<T> = IOkResponse<T> | IErrorResponse

export interface IEndpointTypes {
	oauth: {
		request: {
			code: string
		}
		response: {
			token: string
		}
	}
	profile: {
		request: null
		response: {
			username: string
			token: string
		}
	}
	deployments: {
		request: null
		response: Deployment[]
	}
	[E: `${string}/get_env`]: {
		request: null
		response: Record<string, string>
	}
	[E: `${string}/get_status`]: {
		request: null
		response: {
			container: string
			state:
				| ''
				| 'unknown'
				| 'created'
				| 'restarting'
				| 'running'
				| 'removing'
				| 'paused'
				| 'exited'
				| 'dead'
			status: string
		}[]
	}
	[E: `${string}/stop`]: {
		request: null
		response: null
	}
	[E: `${string}/start`]: {
		request: null
		response: null
	}
	[E: `${string}/logs/${string}`]: {
		request: null
		response: string[]
	}
}
