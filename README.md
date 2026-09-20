# CipherFrame

**Secure video, one chunk at a time.**

CipherFrame is an open-source project written in Rust that explores secure, chunk-based video encryption and controlled playback.

The project is currently in early development. Its long-term goal is to provide a transparent and security-focused system for splitting video files into chunks, encrypting them, and securely processing them for authorized playback.

## Status

🚧 **Early Development**

CipherFrame is currently a learning and research project. The encryption, video processing, key management, and playback systems are not implemented yet.

The architecture will evolve as the project develops.

## Goals

- Split video files into independently manageable chunks
- Encrypt video chunks using established cryptographic standards
- Verify the integrity of encrypted data
- Decrypt content only when authorized
- Minimize unnecessary exposure of decrypted video data
- Develop a secure key-management architecture
- Explore resistance against tampering and unauthorized analysis
- Keep the implementation open source and auditable

## Security Philosophy

CipherFrame does not intend to rely on secret algorithms.

The source code is public so that the implementation can be inspected, reviewed, tested, and improved.

Security should instead depend on properly implemented cryptographic standards, secure key management, authentication, authorization, and careful software design.

CipherFrame will not implement a custom cryptographic algorithm when an established and well-reviewed alternative is appropriate.

## Planned Architecture

```text
                    Video
                      │
                      ▼
                ┌───────────┐
                │  Chunking │
                └─────┬─────┘
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
       Chunk 1     Chunk 2     Chunk N
          │           │           │
          └───────────┼───────────┘
                      ▼
                ┌───────────┐
                │ Encryption│
                └─────┬─────┘
                      │
                      ▼
              Encrypted Chunks
                      │
                      ▼
              Authorization
                      │
                      ▼
                 Decryption
                      │
                      ▼
                  Playback


                  