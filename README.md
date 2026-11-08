# Phidget VoltageRatio Recorder
PhidgetsのVINT Hub（HUB0001）に接続した60 mmスライダーの電圧比（VoltageRatio）を、Windows上で記録するためのシンプルなGUIアプリケーションです。

古い実験用PC（Windows 7 x64）でも利用できるよう、RustとWindows標準のGUI／GDIを使って作成しました。操作に必要な機能を絞り、計測中の値と直近5秒間の波形を確認しながら、CSV形式で保存できます。

## 主な機能

- Phidget22のVoltageRatioInputを用いた約1,000 Hzでのデータ取得（1 msのDataIntervalを設定）
- START／STOPによる計測操作
- VoltageRatioの数値・リアルタイム波形の表示
- 記録モードの選択（計測開始後は変更不可）
  - **Round（標準）**：小数第2位まで四捨五入
  - **Raw**：取得値を加工せず記録
  - **Truncate**：小数第3位以下を切り捨て
- CSVの自動保存（`output/`）

> **注意**：記録する時刻はPC側のコールバック受信時刻です。1 ms周期を設定していますが、各サンプルが厳密に1 ms間隔で取得されることを保証するものではありません。また、Round／Truncateは桁数の調整であり、スライダーの接触不良などに由来する大きなスパイクを除去する機能ではありません。

## 動作環境

- Windows x64
- Phidgets VINT Hub（HUB0001）とVoltageRatioInputとして認識されるスライダー等
- Phidget22のWindows用DLL（`phidget22.dll`）

Windows 7 x64の実験用環境で動作確認していますが、**Phidgets公式の現行Windowsライブラリのサポート対象はWindows 8以降**です。Windows 7での動作は保証されません。Phidgets機器が認識されない場合は、USBドライバの状態も確認してください。

## ダウンロード・使い方

1. GitHubの**Releases**からWindows x64用のZIPをダウンロードして展開します。
2. 下記の手順で公式SDKから`phidget22.dll`を入手し、**EXEと同じフォルダ**へ配置します。
3. HUB0001にスライダーを接続し、HUBをPCに接続します（現在の実装はHub Port 0を使用）。
4. EXEを起動し、記録モードを選択します。通常は`Round`のままで構いません。
5. **START**で計測を開始し、**STOP**で終了するとCSVが保存されます。

### phidget22.dllの入手方法

このリポジトリとReleasesには、Phidgets製のDLLを同梱していません。利用者自身でPhidgets公式サイトから入手してください。

1. [Phidgets公式：Windows向けダウンロード](https://www.phidgets.com/docs/OS_-_Windows)を開きます。
2. **Other Resources → Windows Development Files**からSDKのZIPをダウンロードします。
   - [Windows Development Files（公式ZIPへの直接リンク）](https://cdn.phidgets.com/downloads/phidget22/libraries/windows/Phidget22-windevel.zip)
3. ZIPを展開し、**64-bit用の`phidget22.dll`**を探します。`x86`フォルダ内のファイルは32-bit用なので使用しません。
4. `phidget22.dll`をアプリのEXEと同じディレクトリにコピーしてください。

配置例：

```text
PhidgetVoltageRatioRecorder/
├── <アプリ名>.exe
└── phidget22.dll
```

なお、PhidgetsのWindows用インストーラーを利用する場合、通常は`C:\Program Files\Phidgets\Phidget22\`にもDLLが配置されます。詳しくは[公式のWindows Advanced Information](https://www.phidgets.com/docs/OS_-_Windows_Advanced_Information)を参照してください。

## 保存データ

計測を終了すると、`output/`ディレクトリにCSVファイルを保存します。

```text
output/
├── 20261108_093012_round.csv
├── 20261108_093145_raw.csv
└── 20261108_093308_truncate.csv
```

ファイル名には**計測開始日時と記録モード**を使用します。`output/`の場所は起動時の作業ディレクトリに依存します。

CSVには、サンプル番号、計測開始からの経過時間（秒）、VoltageRatioが記録されます。

## ソースコードからのビルド

Windows上でRustの開発環境を準備し、プロジェクトのルートで実行します。

```powershell
cargo build --release
```

実行ファイルは通常`target/release/`に生成されます。実行時には同じフォルダに`phidget22.dll`を配置してください。Windows 7向けにビルドする際は、Rustおよび依存クレートの対応バージョンに注意してください（開発時の使用ツールチェーン：Rust 1.77.2）。

## ライセンス

![LICENSE: MIT SUSHI-WARE🍣](https://raw.githubusercontent.com/watasuke102/mit-sushi-ware/master/MIT-SUSHI-WARE.svg)

`phidget22.dll`はPhidgets Inc.のソフトウェアであり、このアプリケーションのライセンスの対象外です。 [Phidgets公式のライセンス情報](https://www.phidgets.com/docs/Software_License)を確認してください。
