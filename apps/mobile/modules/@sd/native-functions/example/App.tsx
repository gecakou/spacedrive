import { StyleSheet, Text, View } from 'react-native';

import * as NativeFunctions from '@sd/native-functions';

export default function App() {
  return (
    <View style={styles.container}>
      <Text>{NativeFunctions.hello()}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
});
