export interface TokenCreatedEvent {
  args: {
    creator: string;
    tokenAddress: string;
    name: string;
    symbol: string;
    owner: string;
  };
}
