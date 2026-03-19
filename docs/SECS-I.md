# 데이터 규격
아래 순서로 데이터가 들어옴 (byte 단위)
## 데이터 순서
|Length|Header|Data|checksum|
|-|-|-|-|
|1|10|0~244|2|
- length: header + data의 길이. 최대 254
- header: 고정 10
- data: header 내 정의에 따라 0 ~ 244 가능
- 2 byte (binary checksum)

block =  **Header + Data**, 최대 254 byte

## header 구조
<table border="1" style="border-collapse:collapse;text-align:end;">
  <tr>
    <th> </th>
    <th>8</th>
    <th>7</th>
    <th>6</th>
    <th>5</th>
    <th>4</th>
    <th>3</th>
    <th>2</th>
    <th>1</th>
  </tr>
  <tr>
    <td>1</td>
    <td>R</td>
    <td colspan="7">upper device ID</td>
  </tr>
  <tr>
    <td>2</td>
    <td colspan="8">lower device ID</td>
  </tr>
   <tr>
    <td>3</td>
    <td>W</td>
    <td colspan="7">upper msg ID (Stream)</td>
  </tr>
  <tr>
    <td>4</td>
    <td colspan="8">lower msg ID (Function)</td>
  </tr>
  <tr>
    <td>5</td>
    <td>E</td>
    <td colspan="7">upper block No</td>
  </tr>
  <tr>
    <td>6</td>
    <td colspan="8">lower block No</td>
  </tr>
  <tr>
    <td>7</td>
    <td colspan="8" rowspan="4">system Bytes<br>(transaction ID)</td>
  </tr>
  <tr>
    <td>8</td>
  </tr>
  <tr>
    <td>9</td>
  </tr>
  <tr>
    <td>10</td>
  </tr>
</table>

- R bit: 메시지의 방향 (0: Host → EQP, 1: EQP → Host)
- device ID: 장비를 구분하기 위한 식별자. 항상 통신하는 "장비측" ID로 사용
    - R bit(0): 목적지 = 장비의 ID
    - R bit(1): 출발지 = 장비의 ID
- W bit: Secondary Message 대기 여부. multi block은 같은 값이어야 함
- upper message ID: Stream
- lower message ID: Function
- E bit: 마지막 블록인지? (1 = 마지막 블록)
- block No: 블록 번호. 최대 32767
    - Multi block: 1부터 순서대로. (블록도 순서대로 전송)
    - Single block: 1 or 0 가능
- system byte: 트랜잭션을 위한 ID 값. (device ID + msg 단위)
    - 트랜잭션: primary - secondary 요청·응답
    - 트랜잭션을 구분할 수 있도록 다른 값을 가져야 함
    - primary - secondary는 동일
    - multi block의 경우 모두 동일
    - 트랜잭션마다 1씩 증가하도록 구현하면 적당히 ok



# Block Transfer Protocol
serial line 내 통신 방향을 수립, 메시지 블록을 전달하는 절차  
single byte handshake 기반으로 동작

4개 코드가 사용됨

|명칭| 값| 설명|
|-|-|-|
|ENQ |0b00000101| request to send|
|EOT | 0b00000100|ready to receive|
|ACK |0b00000110|correct reception|
|NAK |0b00010101|incorrect reception|

## timeout
SECS-I에서 설명하는 timeout 자체는 4개이나, Block 수준에서는 2개. 

블록 단위 전송 실패를 감지하기 위한 시간으로, 조정(tune) 가능해야

|종류|명칭|설명|
|-|-|-|
|T1|inter-character timeout|length byte - 2nd checksum byte|
|T2|Protocol Timeout| ENQ ~ EOT<br>EOT ~ length byte<br>2nd checksum byte ~ any char <br> |

- T1: length byte - 2nd checksum byte 안에 있는 byte 간 시간 간격
- T2: 내가 "A"를 송신 후 상대방으로부터 "B"를 수신할 때까지의 시간
- RTY: 전송 실패 최대 반복 횟수

## 전송 로직
아래 조건을 따름
- 데이터 전송 준비 단계(ENQ 전송 후 대기) 중 경합이 발생 시 slave 측이 양보하고 Receive mode로 진입
- 전송 실패 시 RTY 횟수만큼 재전송 시도
- checksum: block(header + data)의 각 byte를 U16으로 더한 값. 오버플로우 허용
- 명세 구현 방법(state machine / event 기반 등) 강제 X
- 명세 상에 언급된 기능은 모두 구현되어야 함

명세 상 언급 된 5개 상태(state)
- IDLE: 초기 상태
- LINE CONTROL: 전송 방향 설정 / 경합 조정 / retry 처리
    - recv ENQ | send EOT | -> RECEIVE 
    - block to send | send ENQ | -> SEND (T2)
    - Slave + ENQ 수신 | send EOT | -> RECEIVE
- SEND
    1. length 1byte + N byte block + 2byte checksum 전송
    2. 송신 측에서 T2 timer 시작 (reply 필요한 경우)
    3. 수신 측에서 checksum 계산 및 응답
        1. T2 내 ACK: 정상 처리됨
        2. NAK or T2 초괴: retry
    4. 
- RECEIVE
    1. length 1byte + N byte block + 2byte checksum 수신
    2. 수신 측에서 checksum 계산 및 응답
    3. timeout 계산, T1 / T2 조건에 맞지 않으면 NAK 전송
    4. length byte가 비정상 or checksum 이상 -> sender이 전송 끝낼 때까지 대기
        - T1과 연계하여 NAK 전송
        - 기존에 받은 데이터 모두 버릴 수 있음
    5. ACK / NAK 전송 후 IDLE 상태로 복귀

- COMPLETION
    1. Receive Completion


## message protocol
1. send
    - 데이터 크기: 단일 블록 최대 244 byte * 최대 블록 수 32767
    - blocking
        - data -> block 전환 시, 모든 block 크기가 동일할 필요 X
        - 성능 측면은 마지막 빼고 꽉 채워서 오는 것이 좋음 (전송 횟수 줄이기 위해)
    - transaction: Primary 송신 -> Secondary(optional) 수신의 과정
    - interleaving
        - 표준 상 다수 트랜잭션의 병행(concurrent) 전송 허용
    - reply linking
        - 마지막 block msg 전송 후 응답 타이머 시작
        - R-bit가 설정된 경우
        - 동일 device ID + 동일 system bytes
    - reply timeout(t3)
        - primary last block 전송 후 - secondary first block 전송 전
        - t3 초과 시 연관된 block list 제거 + transaction 중단
        - reply timer은 transaction 마다 개별
2. receive: 블록들을 식별 및 적절한 메시지로 조립
    - routing error: 요청 - 응답 간 device ID가 다르거나, 알려진 device ID가 아닌 경우 error로 간주
    - duplicate block detection
        - 이전 블록과 완전히 동일한 블록 수신한 경우 버림
        - header(10 byte) 비교, 중복 체크
        - 체크 위해 이전 header 정보 보관 필요
        
