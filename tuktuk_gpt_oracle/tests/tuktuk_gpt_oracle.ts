import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import { TuktukGptOracle } from "../target/types/tuktuk_gpt_oracle";
import { SolanaGptOracle } from "../solana_gpt";
import LLMIdl from "../solana_gpt.json";
import { SystemProgram } from "@solana/web3.js";
import { init, taskKey, taskQueueAuthorityKey } from "@helium/tuktuk-sdk";

describe("tuktuk_gpt_oracle", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.tuktukGptOracle as Program<TuktukGptOracle>;
  const llm_program = new Program(LLMIdl as anchor.Idl, provider) as Program<SolanaGptOracle>;

  const taskQueue = new anchor.web3.PublicKey(
    "2GnWVUxkU1KnwxgTJmwwPV5Vt6uwoJwMMMtAF21sw9cx"
  );

  const queueAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("queue_authority")],
    program.programId
  )[0];

  console.log("taskQueue: ", taskQueue);
  console.log("queueAuthority: ", queueAuthority);
  const taskQueueAuthority = taskQueueAuthorityKey(
    taskQueue,
    queueAuthority
  )[0];

  console.log("taskQueueAuthority: ", taskQueueAuthority);
  async function GetAgentAndInteraction(
    program: Program<TuktukGptOracle>,
    provider: AnchorProvider,
    program_llm: Program<SolanaGptOracle>
  ) {
    const agentAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("agent")],
      program.programId
    )[0];

    const agent = await program.account.agent.fetch(agentAddress);
    console.log("agent: ", agent);
    // Interaction Address
    const interactionAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("interaction"),
        provider.wallet.publicKey.toBuffer(),
        agent.context.toBuffer(),
      ],
      program_llm.programId
    )[0];
    return { agent, agentAddress, interactionAddress };
  }
  xit("InitializeContext!", async () => {
    const counterAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("counter")],
      llm_program.programId
    )[0];
    const counter = await llm_program.account.counter.fetch(counterAddress);
    console.log("counter: ", counter);
    const contextAddress = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("test-context"),
        new anchor.BN(counter.count).toArrayLike(Buffer, "le", 4),
      ],
      llm_program.programId
    )[0];
    const tx = await program.methods
      .initialize()
      .accountsPartial({
        payer: provider.wallet.publicKey,
        llmContext: contextAddress,
        counter: counterAddress,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  xit("interact with agent!", async () => {
    const { agent, agentAddress, interactionAddress } = await GetAgentAndInteraction(program, provider, llm_program);
    const text = "Hello, My name is Namay Gupta. Wassup?";
    const tx = await program.methods.interactAgent(text).accountsPartial({
      payer: provider.wallet.publicKey,
      interaction: interactionAddress,
      agent: agentAddress,
      contextAccount: agent.context,

    }).rpc()
    console.log("Your transaction signature", tx);
  });

  it("ScheduleCallback!", async () => {
    const { agent, agentAddress, interactionAddress } = await GetAgentAndInteraction(
      program,
      provider,
      llm_program
    );
    const tuktuk_prpgram = await init(provider);
    const task_id = 10;
    const text = "Hello, My name is Namay Gupta. Wassup?";
    const tx = await program.methods.schedule(text, task_id).accountsPartial({
      agent: agentAddress,
      contextAccount: agent.context,
      interaction: interactionAddress,
      taskQueue: taskQueue,
      taskQueueAuthority: taskQueueAuthority,
      task: taskKey(taskQueue, task_id)[0],
      queueAuthority: queueAuthority,
      tuktukProgram: tuktuk_prpgram.programId,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).rpc()
    console.log("Your transaction signature", tx);

  });
})
